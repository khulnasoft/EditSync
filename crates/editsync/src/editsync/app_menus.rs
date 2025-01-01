use collab_ui::collab_panel;
use gpui::{Menu, MenuItem, OsAction};
use terminal_view::terminal_panel;

pub fn app_menus() -> Vec<Menu> {
    use editsync_actions::Quit;

    vec![
        Menu {
            name: "Editsync".into(),
            items: vec![
                MenuItem::action("About Editsync…", editsync_actions::About),
                MenuItem::action("Check for Updates", auto_update::Check),
                MenuItem::separator(),
                MenuItem::submenu(Menu {
                    name: "Settings".into(),
                    items: vec![
                        MenuItem::action("Open Settings", super::OpenSettings),
                        MenuItem::action("Open Key Bindings", editsync_actions::OpenKeymap),
                        MenuItem::action("Open Default Settings", super::OpenDefaultSettings),
                        MenuItem::action(
                            "Open Default Key Bindings",
                            editsync_actions::OpenDefaultKeymap,
                        ),
                        MenuItem::action("Open Project Settings", super::OpenProjectSettings),
                        MenuItem::action(
                            "Select Theme...",
                            editsync_actions::theme_selector::Toggle::default(),
                        ),
                    ],
                }),
                MenuItem::separator(),
                MenuItem::submenu(Menu {
                    name: "Services".into(),
                    items: vec![],
                }),
                MenuItem::separator(),
                MenuItem::action("Extensions", editsync_actions::Extensions),
                MenuItem::action("Install CLI", install_cli::Install),
                MenuItem::separator(),
                #[cfg(target_os = "macos")]
                MenuItem::action("Hide Editsync", super::Hide),
                #[cfg(target_os = "macos")]
                MenuItem::action("Hide Others", super::HideOthers),
                #[cfg(target_os = "macos")]
                MenuItem::action("Show All", super::ShowAll),
                MenuItem::action("Quit", Quit),
            ],
        },
        Menu {
            name: "File".into(),
            items: vec![
                MenuItem::action("New", workspace::NewFile),
                MenuItem::action("New Window", workspace::NewWindow),
                MenuItem::separator(),
                MenuItem::action("Open…", workspace::Open),
                MenuItem::action(
                    "Open Recent...",
                    editsync_actions::OpenRecent {
                        create_new_window: true,
                    },
                ),
                MenuItem::separator(),
                MenuItem::action("Add Folder to Project…", workspace::AddFolderToProject),
                MenuItem::action("Save", workspace::Save { save_intent: None }),
                MenuItem::action("Save As…", workspace::SaveAs),
                MenuItem::action("Save All", workspace::SaveAll { save_intent: None }),
                MenuItem::action(
                    "Close Editor",
                    workspace::CloseActiveItem { save_intent: None },
                ),
                MenuItem::action("Close Window", workspace::CloseWindow),
            ],
        },
        Menu {
            name: "Edit".into(),
            items: vec![
                MenuItem::os_action("Undo", editor::actions::Undo, OsAction::Undo),
                MenuItem::os_action("Redo", editor::actions::Redo, OsAction::Redo),
                MenuItem::separator(),
                MenuItem::os_action("Cut", editor::actions::Cut, OsAction::Cut),
                MenuItem::os_action("Copy", editor::actions::Copy, OsAction::Copy),
                MenuItem::os_action("Paste", editor::actions::Paste, OsAction::Paste),
                MenuItem::separator(),
                MenuItem::action("Find", search::buffer_search::Deploy::find()),
                MenuItem::action("Find In Project", workspace::DeploySearch::find()),
                MenuItem::separator(),
                MenuItem::action(
                    "Toggle Line Comment",
                    editor::actions::ToggleComments::default(),
                ),
            ],
        },
        Menu {
            name: "Selection".into(),
            items: vec![
                MenuItem::os_action(
                    "Select All",
                    editor::actions::SelectAll,
                    OsAction::SelectAll,
                ),
                MenuItem::action("Expand Selection", editor::actions::SelectLargerSyntaxNode),
                MenuItem::action("Shrink Selection", editor::actions::SelectSmallerSyntaxNode),
                MenuItem::separator(),
                MenuItem::action("Add Cursor Above", editor::actions::AddSelectionAbove),
                MenuItem::action("Add Cursor Below", editor::actions::AddSelectionBelow),
                MenuItem::action(
                    "Select Next Occurrence",
                    editor::actions::SelectNext {
                        replace_newest: false,
                    },
                ),
                MenuItem::separator(),
                MenuItem::action("Move Line Up", editor::actions::MoveLineUp),
                MenuItem::action("Move Line Down", editor::actions::MoveLineDown),
                MenuItem::action("Duplicate Selection", editor::actions::DuplicateLineDown),
            ],
        },
        Menu {
            name: "View".into(),
            items: vec![
                MenuItem::action("Zoom In", editsync_actions::IncreaseBufferFontSize),
                MenuItem::action("Zoom Out", editsync_actions::DecreaseBufferFontSize),
                MenuItem::action("Reset Zoom", editsync_actions::ResetBufferFontSize),
                MenuItem::separator(),
                MenuItem::action("Toggle Left Dock", workspace::ToggleLeftDock),
                MenuItem::action("Toggle Right Dock", workspace::ToggleRightDock),
                MenuItem::action("Toggle Bottom Dock", workspace::ToggleBottomDock),
                MenuItem::action("Close All Docks", workspace::CloseAllDocks),
                MenuItem::submenu(Menu {
                    name: "Editor Layout".into(),
                    items: vec![
                        MenuItem::action("Split Up", workspace::SplitUp),
                        MenuItem::action("Split Down", workspace::SplitDown),
                        MenuItem::action("Split Left", workspace::SplitLeft),
                        MenuItem::action("Split Right", workspace::SplitRight),
                    ],
                }),
                MenuItem::separator(),
                MenuItem::action("Project Panel", project_panel::ToggleFocus),
                MenuItem::action("Outline Panel", outline_panel::ToggleFocus),
                MenuItem::action("Collab Panel", collab_panel::ToggleFocus),
                MenuItem::action("Terminal Panel", terminal_panel::ToggleFocus),
                MenuItem::separator(),
                MenuItem::action("Diagnostics", diagnostics::Deploy),
                MenuItem::separator(),
            ],
        },
        Menu {
            name: "Go".into(),
            items: vec![
                MenuItem::action("Back", workspace::GoBack),
                MenuItem::action("Forward", workspace::GoForward),
                MenuItem::separator(),
                MenuItem::action("Command Palette...", editsync_actions::command_palette::Toggle),
                MenuItem::separator(),
                MenuItem::action("Go to File...", workspace::ToggleFileFinder::default()),
                // MenuItem::action("Go to Symbol in Project", project_symbols::Toggle),
                MenuItem::action(
                    "Go to Symbol in Editor...",
                    editsync_actions::outline::ToggleOutline,
                ),
                MenuItem::action("Go to Line/Column...", editor::actions::ToggleGoToLine),
                MenuItem::separator(),
                MenuItem::action("Go to Definition", editor::actions::GoToDefinition),
                MenuItem::action("Go to Declaration", editor::actions::GoToDeclaration),
                MenuItem::action("Go to Type Definition", editor::actions::GoToTypeDefinition),
                MenuItem::action("Find All References", editor::actions::FindAllReferences),
                MenuItem::separator(),
                MenuItem::action("Next Problem", editor::actions::GoToDiagnostic),
                MenuItem::action("Previous Problem", editor::actions::GoToPrevDiagnostic),
            ],
        },
        Menu {
            name: "Window".into(),
            items: vec![
                MenuItem::action("Minimize", super::Minimize),
                MenuItem::action("Zoom", super::Zoom),
                MenuItem::separator(),
            ],
        },
        Menu {
            name: "Help".into(),
            items: vec![
                MenuItem::action("View Telemetry", editsync_actions::OpenTelemetryLog),
                MenuItem::action("View Dependency Licenses", editsync_actions::OpenLicenses),
                MenuItem::action("Show Welcome", workspace::Welcome),
                MenuItem::action("Give Feedback...", editsync_actions::feedback::GiveFeedback),
                MenuItem::separator(),
                MenuItem::action(
                    "Documentation",
                    super::OpenBrowser {
                        url: "https://editsync.khulnasoft.com/docs".into(),
                    },
                ),
                MenuItem::action(
                    "Editsync Twitter",
                    super::OpenBrowser {
                        url: "https://twitter.com/editsyncdotdev".into(),
                    },
                ),
                MenuItem::action(
                    "Join the Team",
                    super::OpenBrowser {
                        url: "https://editsync.khulnasoft.com/jobs".into(),
                    },
                ),
            ],
        },
    ]
}