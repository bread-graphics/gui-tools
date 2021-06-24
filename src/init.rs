// MIT/Apache2 License

/// Creates a new `DisplaySum` using the system's resources.
#[inline]
pub(crate) fn init() -> crate::Result<DisplaySum<'_>> {
    let mut last_error = crate::Error::NoInitializer;
    for initializer in INITIALIZERS {
        match initializer() {
            Ok(display) => return Ok(display),
            Err(e) => {
                last_error = e;
            }
        }
    }
    Err(last_error)
}

const INITIALIZERS: &[fn() -> crate::Result<DisplaySum<'_>>] = &[
    #[cfg(unix)]
    || {
        log::info!("Trying to initialize a vanilla \"breadx\" display");
        let mut display = BreadxDisplayConnection::create(None, None)?;

        cfg_if::cfg_if! {
            if #[cfg(feature = "xrender")] {
                log::info!("Attempting to enable xrender...");
                if let Err(e) = display.enable_xrender() {
                    log::error!("Failed to enable xrender, falling back to vanilla impl: {}", e);
                }
            }
        }
        Ok(DisplaySum::Breadx())
    },
    #[cfg(windows)]
    || {
        log::info!("Trying to initialize a \"yaww\" display that uses GDI+ for rendering");
        Ok(DisplaySum::Yaww(YawwDisplay::new()?))
    },
];
