// MIT/Apache2 License

use crate::display::DisplaySum;

#[cfg(unix)]
use crate::breadx::BreadxDisplayConnection;

/// Creates a new `DisplaySum` using the system's resources.
#[inline]
pub(crate) fn init<'evh>() -> crate::Result<DisplaySum<'evh>> {
    let mut last_error = crate::Error::NoInitializer;
    for initializer in Helper::<'evh>::INITIALIZERS {
        match initializer() {
            Ok(display) => return Ok(display),
            Err(e) => {
                last_error = e;
            }
        }
    }
    Err(last_error)
}

/// Parameterize the `INITIALIZERS` set with a lifetime.
struct Helper<'evh>(&'evh ());

impl<'evh> Helper<'evh> {
    const INITIALIZERS: &'evh [fn() -> crate::Result<DisplaySum<'evh>>] = &[
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
            Ok(DisplaySum::Breadx(display))
        },
        #[cfg(windows)]
        || {
            log::info!("Trying to initialize a \"yaww\" display that uses GDI+ for rendering");
            Ok(DisplaySum::Yaww(YawwDisplay::new()?))
        },
    ];
}
