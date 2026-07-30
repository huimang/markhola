use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};

pub(super) struct HiddenExportRuntime {
    application: Retained<NSApplication>,
}

impl HiddenExportRuntime {
    pub(super) fn initialize() -> Result<Self, String> {
        let mtm = MainThreadMarker::new()
            .ok_or_else(|| "Offline export must initialize on the main thread.".to_string())?;
        let application = NSApplication::sharedApplication(mtm);
        if application.activationPolicy() != NSApplicationActivationPolicy::Prohibited
            && (!application.setActivationPolicy(NSApplicationActivationPolicy::Prohibited)
                || application.activationPolicy() != NSApplicationActivationPolicy::Prohibited)
        {
            return Err("The hidden application activation policy could not be set.".to_string());
        }
        if application.windows().count() != 0 {
            return Err("Offline export must not start with attached windows.".to_string());
        }
        Ok(Self { application })
    }

    pub(super) fn finish(self) -> Result<(), String> {
        let windows = self.application.windows();
        if windows.iter().any(|window| window.isVisible()) {
            return Err("Offline export created a visible window.".to_string());
        }
        for window in windows.iter() {
            window.close();
        }
        if self
            .application
            .windows()
            .iter()
            .any(|window| window.isVisible())
        {
            return Err("Offline export could not close its hidden render window.".to_string());
        }
        if self.application.activationPolicy() != NSApplicationActivationPolicy::Prohibited {
            return Err("Offline export changed the hidden activation policy.".to_string());
        }
        Ok(())
    }
}
