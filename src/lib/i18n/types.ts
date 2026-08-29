export interface Translations {
  common: {
    save: string;
    saving: string;
    saved: string;
    cancel: string;
    back: string;
    next: string;
    skip: string;
    close: string;
    edit: string;
    delete: string;
    confirm: string;
    loading: string;
    search: string;
    import: string;
    export: string;
    add: string;
    on: string;
    off: string;
    auto: string;
    yes: string;
    no: string;
    error: string;
    local: string;
    course: string;
    anonymous: string;
    update: string;
    comingSoon: string;
    or: string;
    remove: string;
  };

  locale: {
    label: string;
    en: string;
    it: string;
  };

  nav: {
    timing: string;
    results: string;
    settings: string;
    export: string;
  };

  header: {
    switchToLight: string;
    switchToDark: string;
    changeTheme: string;
    duplicates: string;
  };

  intro: {
    initialSetup: string;
    stepLabels: [string, string, string];
    step0: {
      welcome: string;
      title: string;
      description: string;
      operatorLabel: string;
      operatorPlaceholder: string;
      operatorRequired: string;
    };
    step1: {
      step: string;
      title: string;
      description: string;
    };
    step2: {
      step: string;
      title: string;
      description: string;
    };
    skipConfigureLater: string;
    skipImportLater: string;
    launchWorkspace: string;
  };

  settings: {
    title: string;
    subtitle: string;
    tabs: {
      general: string;
      race: string;
      athletes: string;
      sync: string;
    };
    general: {
      sectionTitle: string;
      operatorName: string;
      dedupWindow: string;
      dedupWarnDelta: string;
    };
    appearance: {
      sectionTitle: string;
      theme: string;
      themeAuto: string;
      themeLight: string;
      themeDark: string;
      themeAutoTitle: string;
      themeLightTitle: string;
      themeDarkTitle: string;
      palette: string;
      paletteStoneTitle: string;
      paletteSlateTitle: string;
      paletteNordTitle: string;
      sound: string;
      soundOnTitle: string;
      soundOffTitle: string;
      language: string;
    };
    race: {
      sectionTitle: string;
      checkpointsSectionTitle: string;
    };
    athletes: {
      sectionTitle: string;
      noAthletes: string;
      searchPlaceholder: string;
      countLabel: string;
      columns: {
        bib: string;
        firstName: string;
        lastName: string;
        category: string;
        course: string;
      };
      anonymous: string;
      localLabel: string;
      confirmDelete: string;
    };
    sync: {
      sectionTitle: string;
      toggleOffLabel: string;
      toggleOffTitle: string;
      toggleOnLabel: string;
      toggleOnTitle: string;
      oidcSection: string;
      apiSection: string;
      authSection: string;
      loginActive: string;
      notAuthenticated: string;
      login: string;
      logout: string;
      offlineDescription: string;
      syncIntervalLabel: string;
    };
    saveConfig: string;
  };

  workspace: {
    noCourses: string;
    addCourseHint: string;
    visibleLanes: string;
    maxLanes: string;
  };

  lane: {
    statusActive: string;
    statusStandby: string;
    statusEnded: string;
    startCourse: string;
    courseEnded: string;
    recordTime: string;
    sameTime: string;
    splitLabel: string;
    splitPlaceholder: string;
    historyHeader: string;
    noFinishes: string;
    pressRecord: string;
    startFirst: string;
    moveToLabel: string;
    newBibHint: string;
    firstNamePlaceholder: string;
    lastNamePlaceholder: string;
    anonBadge: string;
    endCourseTitle: string;
    restartCourseTitle: string;
    restartLabel: string;
    endLabel: string;
    confirmDeleteTime: string;
    confirmUndoFinish: string;
    errorSelectAthlete: string;
    errorNameRequired: string;
    errorInvalidBib: string;
  };

  results: {
    title: string;
    subtitle: string;
    noResults: string;
    courseLabel: string;
    categoryLabel: string;
    allCategories: string;
    searchLabel: string;
    searchPlaceholder: string;
    finishersCount: string;
    columns: {
      pos: string;
      bib: string;
      name: string;
      category: string;
      course: string;
      time: string;
      status: string;
      operator: string;
    };
    statuses: {
      Registered: string;
      Running: string;
      Finished: string;
      Withdrawn: string;
      DNS: string;
    };
    confirmWithdraw: string;
    confirmDns: string;
    withdrawButton: string;
    dnsButton: string;
    anonymous: string;
  };

  export: {
    sectionTitle: string;
    pageTitle: string;
    exportSectionTitle: string;
    exportDescription: string;
    xlsxButton: string;
    csvButton: string;
    backupSectionTitle: string;
    backupDescription: string;
    backupButton: string;
    restoreButton: string;
    restoreConfirmTitle: string;
    restoreConfirmMessage: string;
    restoreConfirmButton: string;
    xlsxSuccess: string;
    csvSuccess: string;
    backupSuccess: string;
    restoreSuccess: string;
    exporting: string;
  };

  modals: {
    confirm: {
      title: string;
      defaultMessage: string;
    };
    confirmRace: {
      endTitle: string;
      endLead: string;
      endBody: string;
      endCta: string;
      endBusy: string;
      restartTitle: string;
      restartLead: string;
      restartBody: string;
      restartCta: string;
      restartBusy: string;
      courseLabel: string;
      typeToConfirm: string;
    };
    assignBib: {
      title: string;
      timestampLabel: string;
      bibAthleteLabel: string;
      confirmLabel: string;
    };
    athleteForm: {
      titleAdd: string;
      titleEdit: string;
      bibLabel: string;
      firstNameLabel: string;
      lastNameLabel: string;
      categoryLabel: string;
      courseLabel: string;
      newCourseOption: string;
      newCourseLabel: string;
      saveButton: string;
      addButton: string;
      errorInvalidBib: string;
      errorNameRequired: string;
      errorCourseRequired: string;
    };
    deviceLogin: {
      title: string;
      loadingCode: string;
      visitUrl: string;
      enterCode: string;
      expiry: string;
      waitingMessage: string;
    };
    duplicateReview: {
      title: string;
      noDuplicates: string;
      bibLabel: string;
      readingsLabel: string;
      refreshButton: string;
    };
  };

  race: {
    raceLabel: string;
    newRaceButton: string;
    deleteRaceButton: string;
    deleteRaceConfirm: string;
    newRaceSectionTitle: string;
    raceNameLabel: string;
    raceNamePlaceholder: string;
    scheduledAtLabel: string;
    createRaceButton: string;
    coursesSectionTitle: string;
    courseNameLabel: string;
    courseNamePlaceholder: string;
    distanceLabel: string;
    addCourseButton: string;
    noCoursesHint: string;
    errorRaceNameRequired: string;
    errorCourseNameRequired: string;
  };

  athletes: {
    importButton: string;
    importing: string;
    addManualButton: string;
    fetchFromServer: string;
    columnHint: string;
    syncLoginHint: string;
    insertedCount: string;
    updatedCount: string;
    coursesCreated: string;
    rowsDiscarded: string;
    rowError: string;
  };

  checkpoints: {
    title: string;
    description: string;
    noCheckpoints: string;
    courseLabel: string;
    newCheckpointLabel: string;
    newCheckpointPlaceholder: string;
    addButton: string;
    removeTitle: string;
    errorCourseRequired: string;
    errorNameRequired: string;
  };
}
