use serde::{Deserialize, Serialize};
use voxine::{cam_controller::CameraConfig, config::EngineConfig};

#[derive(Clone)]
pub struct LiveConfig {
    pub full_detail_distance: f32,
    pub full_detail_generation_distance: f32,
    pub task_cancelation_lod_threshold: voxine::Lod,
    pub render_distance: f32,
    pub max_chunks: usize,

    pub print_tps_per: Option<f64>,
    pub target_tps: f64,

    pub camera: CameraConfig,
    pub gpu_mesh_upload_time: f64,
}
impl voxine::config_loader::Live for LiveConfig {}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigFile {
    pub full_detail_distance: f32,
    pub full_detail_generation_distance: f32,
    pub task_cancelation_lod_threshold: voxine::Lod,
    pub render_distance: f32,
    pub max_chunks: usize,

    pub print_tps: bool,
    pub print_tps_per: Option<f64>,
    pub target_tps: f64,

    pub camera: CameraConfig,
    pub gpu_mesh_upload_time: f64,

    pub starting_pos: [f32; 3],
    pub fov: f32,
    pub near_plane: f32,

    pub worker_count: usize,

    pub task_queue_cap: usize,
    pub engine_worker_config_queue_cap: usize,
    pub discarded_tasks_queue_cap: usize,
    pub mesh_queue_cap: usize,
    pub chunk_queue_cap: usize,
    pub collider_queue_cap: usize,
    pub solid_map_queue_cap: usize,
    pub config_sender_cap: usize,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub full_detail_distance: f32,
    pub full_detail_generation_distance: f32,
    pub task_cancelation_lod_threshold: voxine::Lod,
    pub render_distance: f32,
    pub max_chunks: usize,

    pub print_tps_per: Option<f64>,
    pub target_tps: f64,

    pub camera: CameraConfig,
    pub gpu_mesh_upload_time: f64,

    pub starting_pos: [f32; 3],
    pub fov: f32,
    pub near_plane: f32,

    pub worker_count: usize,

    pub task_queue_cap: usize,
    pub engine_worker_config_queue_cap: usize,
    pub discarded_tasks_queue_cap: usize,
    pub mesh_queue_cap: usize,
    pub chunk_queue_cap: usize,
    pub collider_queue_cap: usize,
    pub solid_map_queue_cap: usize,
    pub config_sender_cap: usize,
}

pub struct Error {
    msg: String,
}

impl From<Error> for voxine::error::ConfigError {
    fn from(value: Error) -> Self {
        voxine::error::ConfigError::LogicError { msg: value.msg }
    }
}

impl voxine::config_loader::ConfigFile<LiveConfig, Config, Error> for ConfigFile {
    fn check(self) -> Result<Config, Error> {
        let ConfigFile {
            full_detail_distance,
            full_detail_generation_distance,
            task_cancelation_lod_threshold,
            render_distance,
            max_chunks,
            print_tps,
            print_tps_per,
            target_tps,
            camera,
            gpu_mesh_upload_time,
            starting_pos,
            fov,
            near_plane,
            worker_count,
            task_queue_cap,
            engine_worker_config_queue_cap,
            discarded_tasks_queue_cap,
            mesh_queue_cap,
            chunk_queue_cap,
            collider_queue_cap,
            solid_map_queue_cap,
            config_sender_cap,
        } = self;

        Ok(Config {
            full_detail_distance,
            full_detail_generation_distance,
            task_cancelation_lod_threshold,

            render_distance,

            max_chunks,

            print_tps_per: if print_tps {
                match print_tps_per {
                    Some(print_tps_per) => Some(print_tps_per),
                    None => {
                        return Err(Error {
                            msg: "print-tps is activated but not print-tps-per".to_string(),
                        });
                    }
                }
            } else {
                None
            },
            target_tps,

            camera,
            gpu_mesh_upload_time,
            starting_pos,
            fov,
            near_plane,

            worker_count: worker_count.min(num_cpus::get()),

            task_queue_cap,
            engine_worker_config_queue_cap,
            discarded_tasks_queue_cap,
            mesh_queue_cap,
            chunk_queue_cap,
            collider_queue_cap,
            solid_map_queue_cap,
            config_sender_cap,
        })
    }
}

impl voxine::config_loader::Config<LiveConfig> for Config {
    fn live(self) -> LiveConfig {
        let Config {
            starting_pos: _,
            fov: _,
            near_plane: _,
            worker_count: _,
            task_queue_cap: _,
            engine_worker_config_queue_cap: _,
            discarded_tasks_queue_cap: _,
            mesh_queue_cap: _,
            chunk_queue_cap: _,
            collider_queue_cap: _,
            solid_map_queue_cap: _,
            config_sender_cap: _,

            full_detail_distance,
            full_detail_generation_distance,
            task_cancelation_lod_threshold,
            render_distance,
            max_chunks,
            print_tps_per,
            target_tps,
            camera,
            gpu_mesh_upload_time,
        } = self;

        LiveConfig {
            full_detail_distance,
            full_detail_generation_distance,
            task_cancelation_lod_threshold,
            render_distance,
            max_chunks,

            print_tps_per,
            target_tps,

            camera,
            gpu_mesh_upload_time,
        }
    }

    fn sender_cap(&self) -> usize {
        self.config_sender_cap
    }
}

impl Config {
    pub fn update(&mut self, update: LiveConfig) {
        let LiveConfig {
            full_detail_distance,
            full_detail_generation_distance,
            task_cancelation_lod_threshold,
            render_distance,
            max_chunks,
            print_tps_per,
            target_tps,
            camera,
            gpu_mesh_upload_time,
        } = update;

        self.full_detail_distance = full_detail_distance;
        self.full_detail_generation_distance = full_detail_generation_distance;
        self.task_cancelation_lod_threshold = task_cancelation_lod_threshold;
        self.render_distance = render_distance;
        self.max_chunks = max_chunks;
        self.print_tps_per = print_tps_per;
        self.target_tps = target_tps;
        self.camera = camera;
        self.gpu_mesh_upload_time = gpu_mesh_upload_time;
    }

    pub fn engine_config(self) -> EngineConfig {
        let Config {
            config_sender_cap: _,

            full_detail_distance: _,
            camera: _,
            gpu_mesh_upload_time: _,
            starting_pos: _,
            fov: _,
            near_plane: _,

            full_detail_generation_distance,
            task_cancelation_lod_threshold,
            render_distance,
            max_chunks,
            print_tps_per,
            target_tps,
            worker_count,
            task_queue_cap,
            engine_worker_config_queue_cap,
            discarded_tasks_queue_cap,
            mesh_queue_cap,
            chunk_queue_cap,
            collider_queue_cap,
            solid_map_queue_cap,
        } = self;

        EngineConfig {
            full_detail_distance: full_detail_generation_distance / 32.,
            task_cancelation_lod_threshold,
            total_generation_distance: render_distance / 32.,
            max_chunks,

            print_tps_per,
            target_tps,

            worker_count,

            task_queue_cap,
            engine_worker_config_queue_cap,
            discarded_tasks_queue_cap,
            mesh_queue_cap,
            chunk_queue_cap,
            collider_queue_cap,
            solid_map_queue_cap,
        }
    }
}

impl LiveConfig {
    pub fn engine_config_update(self) -> voxine::config::ConfigUpdate {
        let LiveConfig {
            full_detail_distance: _,
            camera: _,
            gpu_mesh_upload_time: _,

            full_detail_generation_distance,
            task_cancelation_lod_threshold,
            render_distance,
            max_chunks,
            print_tps_per,
            target_tps,
        } = self;

        voxine::config::ConfigUpdate {
            full_detail_distance: full_detail_generation_distance / 32.,
            task_cancelation_lod_threshold,
            total_generation_distance: render_distance / 32.,
            max_chunks,

            print_tps_per: print_tps_per,
            target_tps,
        }
    }
}
