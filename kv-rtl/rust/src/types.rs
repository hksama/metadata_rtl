//! Shared widths and encodings — must match `rtl/kv_pkg.sv` defaults.

pub struct Widths {
    pub page_id: u32,
    pub request_id: u32,
    pub counter: u32,
    pub timestamp: u32,
    pub refcount: u32,
    pub score: u32,
    pub aging_counter: u32,
}

pub const NUM_PAGES: usize = 16;
pub const DEFAULT_SCORE_WIDTH: u32 = 8;
pub const DEFAULT_AGING_INTERVAL: u32 = 64;

pub const WIDTHS: Widths = Widths {
    page_id: 16,
    request_id: 16,
    counter: 16,
    timestamp: 16,
    refcount: 8,
    score: 8,
    aging_counter: 4,
};

pub const COUNTER_MAX: u16 = u16::MAX;
pub const REFCOUNT_MAX: u8 = u8::MAX;
pub const TIMESTAMP_MAX: u16 = u16::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HotnessPolicy {
    Frequency = 0,
    Aging2Bit = 1,
    Aging4Bit = 2,
    ApproxLru = 3,
}

impl HotnessPolicy {
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_uppercase().as_str() {
            "FREQUENCY" | "FREQ" => Some(Self::Frequency),
            "AGING2" | "AGING_2BIT" => Some(Self::Aging2Bit),
            "AGING4" | "AGING_4BIT" => Some(Self::Aging4Bit),
            "LRU" | "APPROX_LRU" => Some(Self::ApproxLru),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Frequency => "FREQUENCY",
            Self::Aging2Bit => "AGING2",
            Self::Aging4Bit => "AGING4",
            Self::ApproxLru => "LRU",
        }
    }

    pub fn all() -> [Self; 4] {
        [
            Self::Frequency,
            Self::Aging2Bit,
            Self::Aging4Bit,
            Self::ApproxLru,
        ]
    }

    pub fn aging_counter_bits(self) -> u32 {
        match self {
            Self::Aging2Bit => 2,
            Self::Aging4Bit => 4,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Operation {
    Read = 0,
    Write = 1,
    Release = 2,
    Allocate = 3,
    Access = 4,
}

impl Operation {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Read),
            1 => Some(Self::Write),
            2 => Some(Self::Release),
            3 => Some(Self::Allocate),
            4 => Some(Self::Access),
            _ => None,
        }
    }

    pub fn is_access(self) -> bool {
        matches!(self, Self::Read | Self::Write | Self::Access)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PageState {
    Invalid = 0,
    Valid = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccessEvent {
    pub request_id: u16,
    pub page_id: u16,
    pub timestamp: u16,
    pub operation: Operation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageMetadata {
    pub page_id: u16,
    pub request_id: u16,
    pub access_count: u16,
    pub last_access: u16,
    pub refcount: u8,
    pub valid: bool,
    pub state: PageState,
    pub aging_counter: u8,
    pub hotness_score: u8,
}

impl PageMetadata {
    pub const INVALID: Self = Self {
        page_id: 0,
        request_id: 0,
        access_count: 0,
        last_access: 0,
        refcount: 0,
        valid: false,
        state: PageState::Invalid,
        aging_counter: 0,
        hotness_score: 0,
    };
}

pub fn sat_inc_u16(value: u16) -> u16 {
    value.saturating_add(1)
}

pub fn sat_inc_u8(value: u8) -> u8 {
    value.saturating_add(1)
}

pub fn sat_dec_u8(value: u8) -> u8 {
    value.saturating_sub(1)
}
