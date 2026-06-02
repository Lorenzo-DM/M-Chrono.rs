// Mock backend for local testing.
//
// Run: bun run scripts/mock-server.ts
// Serves on http://localhost:8787
//
// Endpoints:
//   POST /oauth/v2/device_authorization   — returns fixed user_code "ABCD-1234"
//   POST /oauth/v2/token                  — device_code grant: auto-approves after 1 poll
//                                          refresh_token grant: always returns new access_token
//   GET  /courses                         — fake course list (Bearer required)
//   GET  /athletes                        — fake athlete list (Bearer required)
//   POST /timings/batch                   — echo with assigned remote_ids
//   POST /pending_finishes/batch          — echo with assigned remote_ids
//   GET  /timings?since=<id>&limit=<n>    — empty (no other-operator data; tweak below)
//
// Configure the app to point here:
//   oidc_issuer_url = "http://localhost:8787"
//   oidc_client_id  = "mock-client"
//   api_base_url    = "http://localhost:8787"
//
// The user_code shown by the app should be "ABCD-1234" but the actual content
// does not matter — the mock auto-succeeds on the SECOND poll regardless.

type Course = {
  id: number; name: string; distance_m: number | null;
  started_at_ms: number | null; scheduled_at_ms: number | null;
};
type Athlete = {
  id: number; bib_number: number; first_name: string; last_name: string; course_id: number;
};

const COURSES: Course[] = [
  { id: 1, name: "21K", distance_m: 21_000, started_at_ms: null, scheduled_at_ms: null },
  { id: 2, name: "42K", distance_m: 42_000, started_at_ms: null, scheduled_at_ms: null },
];

const FIRST_NAMES = [
  "Mario", "Luigi", "Giulia", "Anna", "Marco", "Luca", "Sara", "Elena",
  "Paolo", "Chiara", "Andrea", "Francesca", "Giovanni", "Alessia",
  "Stefano", "Martina", "Davide", "Federica", "Roberto", "Valentina",
];
const LAST_NAMES = [
  "Rossi", "Bianchi", "Romano", "Ricci", "Marino", "Greco", "Bruno",
  "Gallo", "Conti", "De Luca", "Mancini", "Costa", "Giordano", "Rizzo",
  "Lombardi", "Moretti", "Barbieri", "Fontana", "Santoro", "Mariani",
];

const ATHLETES: Athlete[] = Array.from({ length: 60 }, (_, i): Athlete => {
  const id = i + 1;
  const bib = 100 + id;
  return {
    id,
    bib_number: bib,
    first_name: FIRST_NAMES[i % FIRST_NAMES.length],
    last_name: LAST_NAMES[(i * 7) % LAST_NAMES.length],
    course_id: i < 30 ? 1 : 2,
  };
});

// poll attempt counter, keyed by device_code
const DEVICE_POLL_COUNT = new Map<string, number>();
// fake remote_id counter
let NEXT_REMOTE_ID = 1;

function json(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json" },
  });
}

function requireBearer(req: Request): string | null {
  const auth = req.headers.get("authorization");
  if (!auth || !auth.startsWith("Bearer ")) return null;
  return auth.slice(7);
}

async function parseForm(req: Request): Promise<Record<string, string>> {
  const text = await req.text();
  const params = new URLSearchParams(text);
  const out: Record<string, string> = {};
  for (const [k, v] of params) out[k] = v;
  return out;
}

const server = Bun.serve({
  port: 8787,
  async fetch(req) {
    const url = new URL(req.url);
    const path = url.pathname;
    const method = req.method;

    console.log(`${method} ${path}`);

    // OIDC: device authorization
    if (method === "POST" && path === "/oauth/v2/device_authorization") {
      const form = await parseForm(req);
      const device_code = `dev-${Math.random().toString(36).slice(2)}`;
      DEVICE_POLL_COUNT.set(device_code, 0);
      return json(200, {
        device_code,
        user_code: "ABCD-1234",
        verification_uri: "http://localhost:8787/verify",
        verification_uri_complete: "http://localhost:8787/verify?code=ABCD-1234",
        expires_in: 600,
        interval: 2,
        scope: form["scope"] ?? "",
      });
    }

    // OIDC: token (device_code grant + refresh_token grant)
    if (method === "POST" && path === "/oauth/v2/token") {
      const form = await parseForm(req);
      const grant = form["grant_type"];

      if (grant === "urn:ietf:params:oauth:grant-type:device_code") {
        const dc = form["device_code"];
        const n = (DEVICE_POLL_COUNT.get(dc) ?? 0) + 1;
        DEVICE_POLL_COUNT.set(dc, n);
        // First poll => pending; second poll => success
        if (n < 2) {
          return json(400, { error: "authorization_pending" });
        }
        return json(200, {
          access_token: `at-${Date.now()}`,
          refresh_token: `rt-${Math.random().toString(36).slice(2)}`,
          expires_in: 3600,
          token_type: "Bearer",
        });
      }

      if (grant === "refresh_token") {
        return json(200, {
          access_token: `at-${Date.now()}`,
          refresh_token: form["refresh_token"], // do not rotate
          expires_in: 3600,
          token_type: "Bearer",
        });
      }

      return json(400, { error: "unsupported_grant_type" });
    }

    // Helpful landing page for the verification URL
    if (method === "GET" && path === "/verify") {
      return new Response(
        `<html><body style="font-family: sans-serif; background:#000; color:#fff; padding:2rem">
           <h1>Mock OIDC verification</h1>
           <p>Code received. Authorization granted automatically.</p>
           <p>Return to the app — it should switch to logged-in state shortly.</p>
         </body></html>`,
        { headers: { "content-type": "text/html" } },
      );
    }

    // API endpoints — all require Bearer token
    const tok = requireBearer(req);
    if (!tok) {
      return json(401, { error: "unauthorized" });
    }

    if (method === "GET" && path === "/courses") {
      return json(200, COURSES);
    }
    if (method === "GET" && path === "/athletes") {
      return json(200, ATHLETES);
    }

    if (method === "POST" && path === "/timings/batch") {
      const body = await req.json() as Array<{ local_id: number }>;
      const acks = body.map(t => ({ local_id: t.local_id, remote_id: NEXT_REMOTE_ID++ }));
      return json(200, acks);
    }

    if (method === "POST" && path === "/pending_finishes/batch") {
      const body = await req.json() as Array<{ local_id: number }>;
      const acks = body.map(p => ({ local_id: p.local_id, remote_id: NEXT_REMOTE_ID++ }));
      return json(200, acks);
    }

    if (method === "GET" && path === "/timings") {
      // No other-operator data for now. Toy with this to simulate sync pull.
      return json(200, []);
    }

    return json(404, { error: "not_found", path });
  },
});

console.log(`mock server listening on http://localhost:${server.port}`);
console.log(`courses=${COURSES.length} athletes=${ATHLETES.length}`);
console.log(`stop with Ctrl-C`);
