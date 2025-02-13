use release_channel::ReleaseChannel;
use windows::{
    core::HSTRING,
    Win32::{
        Foundation::{GetLastError, ERROR_ALREADY_EXISTS},
        System::Threading::CreateEventW,
    },
};

fn retrieve_app_instance_event_identifier() -> &'static str {
    match *release_channel::RELEASE_CHANNEL {
        ReleaseChannel::Dev => "Local\\Editsync-Editor-Dev-Instance-Event",
        ReleaseChannel::Nightly => "Local\\Editsync-Editor-Nightly-Instance-Event",
        ReleaseChannel::Preview => "Local\\Editsync-Editor-Preview-Instance-Event",
        ReleaseChannel::Stable => "Local\\Editsync-Editor-Stable-Instance-Event",
    }
}

pub fn check_single_instance() -> bool {
    unsafe {
        CreateEventW(
            None,
            false,
            false,
            &HSTRING::from(retrieve_app_instance_event_identifier()),
        )
        .expect("Unable to create instance sync event")
    };
    let last_err = unsafe { GetLastError() };
    last_err != ERROR_ALREADY_EXISTS
}
