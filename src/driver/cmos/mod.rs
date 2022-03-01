use x86_64::instructions::port::Port;

const PORT_REGISTER_SELECT: u16 = 0x70;
const PORT_DATA: u16 = 0x71;

const REGISTER_RTC_SECONDS: u8 = 0x00;
const REGISTER_RTC_MINUTES: u8 = 0x02;
const REGISTER_RTC_HOURS: u8 = 0x04;
const REGISTER_RTC_WEEKDAY: u8 = 0x06;
const REGISTER_RTC_DAY_OF_MONTH: u8 = 0x07;
const REGISTER_RTC_MONTH: u8 = 0x08;
const REGISTER_RTC_YEAR: u8 = 0x09;
const REGISTER_RTC_CENTURY: u8 = 0x32;
const REGISTER_RTC_STATUS_A: u8 = 0x0A;
const REGISTER_RTC_STATUS_B: u8 = 0x0B;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct CMOSTime {
    pub seconds: u8,
    pub minutes: u8,
    pub hours: u8,
    pub weekday: u8,
    pub day_of_month: u8,
    pub month: u8,
    pub year: u8,
    pub century: Option<u8>,
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
enum Mode {
    Binary,
    BCD,
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
enum HourFormat {
    _24h,
    _12h,
}

pub struct CMOS {
    initialized: bool,
    mode: Mode,
    hour_format: HourFormat,
    register_select: Port<u8>,
    data: Port<u8>,
}

impl CMOS {
    pub const fn new() -> Self {
        Self {
            initialized: false,
            mode: Mode::BCD,
            hour_format: HourFormat::_12h,

            register_select: Port::new(PORT_REGISTER_SELECT),
            data: Port::new(PORT_DATA),
        }
    }

    fn init(&mut self) {
        let status_b = self.read_register(REGISTER_RTC_STATUS_B);
        if (status_b & (1 << 1)) > 0 {
            self.hour_format = HourFormat::_24h;
        }
        if (status_b & (1 << 2)) > 0 {
            self.mode = Mode::Binary;
        }
    }

    pub fn read_time(&mut self) -> CMOSTime {
        let mut first: CMOSTime;
        let mut second: CMOSTime;
        while self.update_in_progress() {
            // wait
        }
        first = self.read_cmos_time_raw();
        loop {
            while self.update_in_progress() {
                // wait again
            }
            second = self.read_cmos_time_raw();
            if first == second {
                return second;
            }
            first = second;
        }
    }

    fn read_cmos_time_raw(&mut self) -> CMOSTime {
        if !self.initialized {
            self.init();
        }

        let mut seconds = self.read_register(REGISTER_RTC_SECONDS);
        let mut minutes = self.read_register(REGISTER_RTC_MINUTES);
        let mut hours = self.read_register(REGISTER_RTC_HOURS);
        let mut day_of_month = self.read_register(REGISTER_RTC_DAY_OF_MONTH);
        let mut month = self.read_register(REGISTER_RTC_MONTH);
        let mut year = self.read_register(REGISTER_RTC_YEAR);
        let mut century = self.read_register(REGISTER_RTC_CENTURY);
        if self.mode == Mode::BCD {
            // we have to convert the values to binary
            seconds = bcd_to_binary(seconds);
            minutes = bcd_to_binary(minutes);
            hours = bcd_to_binary(hours);
            day_of_month = bcd_to_binary(day_of_month);
            month = bcd_to_binary(month);
            year = bcd_to_binary(year);
            century = bcd_to_binary(century);
        }
        let weekday = self.read_register(REGISTER_RTC_WEEKDAY);

        CMOSTime {
            seconds,
            minutes,
            hours,
            weekday,
            day_of_month,
            month,
            year,
            century: Some(century),
        }
    }

    pub fn read_register(&mut self, register: u8) -> u8 {
        self.select_register(register);
        // TODO: reasonable delay
        self.read_data()
    }

    pub fn write_register(&mut self, register: u8, value: u8) {
        self.select_register(register);
        // TODO: reasonable delay
        self.write_data(value);
    }

    fn select_register(&mut self, register: u8) {
        unsafe {
            self.register_select.write(register);
        }
    }

    fn write_data(&mut self, value: u8) {
        unsafe {
            self.data.write(value);
        }
    }

    fn read_data(&mut self) -> u8 {
        unsafe { self.data.read() }
    }

    fn update_in_progress(&mut self) -> bool {
        self.read_register(REGISTER_RTC_STATUS_A) & (1 << 6) > 0
    }
}

fn bcd_to_binary(bcd: u8) -> u8 {
    ((bcd & 0xF0) >> 1) + ((bcd & 0xF0) >> 3) + (bcd & 0x0F)
}
