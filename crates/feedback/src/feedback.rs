use gpui::{actions, AppContext, ClipboardItem, PromptLevel};
use system_specs::SystemSpecs;
use util::ResultExt;
use workspace::Workspace;

pub mod feedback_modal;

mod system_specs;

actions!(
    editsync,
    [
        CopySystemSpecsIntoClipboard,
        FileBugReport,
        RequestFeature,
        OpenEditsyncRepo
    ]
);

const fn editsync_repo_url() -> &'static str {
    "https://github.com/khulnasoft/editsync"
}

fn request_feature_url(specs: &SystemSpecs) -> String {
    format!(
        concat!(
            "https://github.com/khulnasoft/editsync/issues/new",
            "?labels=admin+read%2Ctriage%2Cenhancement",
            "&template=0_feature_request.yml",
            "&environment={}"
        ),
        urlencoding::encode(&specs.to_string())
    )
}

fn file_bug_report_url(specs: &SystemSpecs) -> String {
    format!(
        concat!(
            "https://github.com/khulnasoft/editsync/issues/new",
            "?labels=admin+read%2Ctriage%2Cbug",
            "&template=1_bug_report.yml",
            "&environment={}"
        ),
        urlencoding::encode(&specs.to_string())
    )
}

pub fn init(cx: &mut AppContext) {
    cx.observe_new_views(|workspace: &mut Workspace, cx| {
        feedback_modal::FeedbackModal::register(workspace, cx);
        workspace
            .register_action(|_, _: &CopySystemSpecsIntoClipboard, cx| {
                let specs = SystemSpecs::new(cx);

                cx.spawn(|_, mut cx| async move {
                    let specs = specs.await.to_string();

                    cx.update(|cx| cx.write_to_clipboard(ClipboardItem::new_string(specs.clone())))
                        .log_err();

                    cx.prompt(
                        PromptLevel::Info,
                        "Copied into clipboard",
                        Some(&specs),
                        &["OK"],
                    )
                    .await
                    .ok();
                })
                .detach();
            })
            .register_action(|_, _: &RequestFeature, cx| {
                let specs = SystemSpecs::new(cx);
                cx.spawn(|_, mut cx| async move {
                    let specs = specs.await;
                    cx.update(|cx| {
                        cx.open_url(&request_feature_url(&specs));
                    })
                    .log_err();
                })
                .detach();
            })
            .register_action(move |_, _: &FileBugReport, cx| {
                let specs = SystemSpecs::new(cx);
                cx.spawn(|_, mut cx| async move {
                    let specs = specs.await;
                    cx.update(|cx| {
                        cx.open_url(&file_bug_report_url(&specs));
                    })
                    .log_err();
                })
                .detach();
            })
            .register_action(move |_, _: &OpenEditsyncRepo, cx| {
                cx.open_url(editsync_repo_url());
            });
    })
    .detach();
}
