use serde::{Deserialize, Serialize};

fn default_capture_speed() -> f32 {
    1.0
}
#[derive(Serialize, Deserialize, Debug, Copy, Default, Clone)]
pub struct CapturePoint {
    pub x: f32,
    pub y: f32,
    #[serde(default)]
    pub yaw: f32,
    pub radius: f32,
    #[serde(default = "default_capture_speed")]
    pub capture_speed: f32,
    #[serde(default)]
    pub team: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(tag = "type")]
pub enum MatchType {
    #[default]
    None,
    Domination {
        team_deathmatch_min: i64,
        point_limit: Option<f32>,
        capture_points: Vec<CapturePoint>,
    },
    TeamDeathmatch {
        point_limit: Option<i64>,
    },
    KingOfTheHill {
        capture_points: Vec<CapturePoint>,
        point_limit: Option<f32>,
    },
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct MatchConfig {
    #[serde(default)]
    pub mode: MatchType,
    pub time_limit: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Team {
    /// Team name
    pub name: String,

    /// Team comment, like controller that was loaded.
    pub comment: Option<String>,

    /// Color used to represent this team. RGB; 0-255.
    pub color: (u8, u8, u8),

    /// The controller to use for this team.
    pub controller: Option<ControllerType>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct WasmControlConfig {
    pub path: String,
    #[serde(default)]
    pub fuel_per_update: Option<u64>,
    #[serde(default)]
    pub fuel_for_setup: Option<u64>,
    #[serde(default)]
    pub reload: bool,
}
impl Default for WasmControlConfig {
    fn default() -> Self {
        Self {
            path: "".to_owned(),
            fuel_per_update: None,
            fuel_for_setup: None,
            reload: true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum ControllerType {
    #[default]
    Idle,
    SwivelShoot,
    RadioPosition,
    #[cfg(not(target_arch = "wasm32"))]
    LibraryLoad {
        name: String,
    },
    DiffDriveForwardsBackwards {
        velocities: (f32, f32),
        duration: f32,
    },
    DiffDriveCapturable,
    InterfacePrinter,
    NaiveShoot,
    #[cfg(feature = "unit_control_wasm")]
    Wasm(WasmControlConfig),
    #[serde(skip)]
    Function(battleground_unit_control::ControllerSpawn),
    SequenceControl {
        controllers: Vec<ControllerType>,
    },
    FromControlConfig {
        name: String,
    },
    TeamController {
        name: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Copy, Default, Clone)]
pub enum Unit {
    #[default]
    Tank,
    Artillery,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Spawn {
    pub team: Option<usize>,
    #[serde(default)]
    pub unit: Unit,
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
    #[serde(default)]
    pub controller: ControllerType,
    #[serde(default)]
    pub radio: crate::units::common::RadioConfig,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SpawnConfig {
    #[serde(default)]
    pub control_config: std::collections::HashMap<String, ControllerType>,
    pub teams: Vec<Team>,
    pub spawns: Vec<Spawn>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ScenarioConfig {
    /// String used to invoke special setup.
    #[serde(default)]
    pub pre_setup: String,

    #[serde(default)]
    pub recording: bool,

    /// Denotes the match specification.
    #[serde(default)]
    pub match_config: MatchConfig,

    /// Spawn of vehicles.
    #[serde(default)]
    pub spawn_config: SpawnConfig,
}

/// This struct specifies the steps to be done after a scenario wraps up.
pub struct WrapUpConfig {
    /// Additional time to run update before writing recording after wrap up is called.
    pub outro: f32,

    /// Write the wrap up report to this file if a path is specified.
    pub write_wrap_up: Option<String>,

    /// Write the recording file to this path if specified.
    pub write_recording: Option<String>,

    /// The original scenario as setup.
    pub scenario: Option<ScenarioConfig>,
}
