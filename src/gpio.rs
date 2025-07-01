use esp_idf_hal::gpio::*;

/// Generic GPIO output wrapper using type-erased `AnyOutputPin`
pub struct GpioOut {
    pin: PinDriver<'static, AnyOutputPin, Output>,
}

impl GpioOut {
    /// Initialize GPIO output from any pin implementing `Into<AnyOutputPin>`
    pub fn new<P>(pin: P) -> Result<Self, &'static str>
    where
        P: Into<AnyOutputPin> + 'static,
    {
        let erased = pin.into();
        let driver = PinDriver::output(erased).map_err(|_| "Failed to initialize output")?;
        Ok(GpioOut { pin: driver })
    }

    pub fn set_high(&mut self) -> Result<(), &'static str> {
        self.pin.set_high().map_err(|_| "Failed to set high")
    }

    pub fn set_low(&mut self) -> Result<(), &'static str> {
        self.pin.set_low().map_err(|_| "Failed to set low")
    }

    pub fn toggle(&mut self) -> Result<(), &'static str> {
        self.pin.toggle().map_err(|_| "Failed to toggle")
    }
}
