use strum_macros::EnumIter;

#[derive(EnumIter, Debug, Clone)]
pub enum OptLevel {
    S,
    Z,
    Three,
}
impl OptLevel {
    pub fn option(&self) -> String {
        match self {
            Self::S => "opt-level = \"s\"".to_string(),
            Self::Z => "opt-level = \"z\"".to_string(),
            Self::Three => "opt-level = \"3\"".to_string(),
        }
    }
}

#[derive(EnumIter, Debug, Clone)]
pub enum Lto {
    Off,
    Thin,
    Fat,
}
impl Lto {
    pub fn option(&self) -> String {
        match self {
            Self::Off => "lto = \"off\"".to_string(),
            Self::Thin => "lto = \"thin\"".to_string(),
            Self::Fat => "lto = \"fat\"".to_string(),
        }
    }
}

#[derive(EnumIter, Debug, Clone)]
pub enum CodegenUnits {
    One,
    Default,
}
impl CodegenUnits {
    pub fn option(&self) -> String {
        match self {
            Self::One => "codegen-units = 1".to_string(),
            Self::Default => "".to_string(),
        }
    }
}

#[derive(EnumIter, Debug, Clone)]
pub enum Strip {
    None,
    DebugInfo,
}
impl Strip {
    pub fn option(&self) -> String {
        match self {
            Self::None => "".to_string(),
            Self::DebugInfo => "strip = \"debuginfo\"".to_string(),
        }
    }
}

#[derive(EnumIter, Debug)]
pub enum WasmOpt {
    None,
    S,
    Z,
    Three,
    Both,
}
impl WasmOpt {
    pub fn enabled(&self) -> bool {
        !matches!(self, Self::None)
    }
    pub fn args(&self) -> Vec<String> {
        match self {
            Self::None => vec![],
            Self::S => vec!["-Os".to_string()],
            Self::Z => vec!["-Oz".to_string()],
            Self::Three => vec!["-O3".to_string()],
            Self::Both => vec![
                "-O".to_string(),
                "-s".to_string(),
                "100".to_string(),
                "-ol".to_string(),
                "100".to_string(),
            ],
        }
    }
}

#[derive(EnumIter, Debug)]
pub enum Panic {
    Unwind,
    Abort,
}
impl Panic {
    pub fn option(&self) -> String {
        match self {
            Self::Unwind => "panic = \"unwind\"".to_string(),
            Self::Abort => "panic = \"abort\"".to_string(),
        }
    }
}
