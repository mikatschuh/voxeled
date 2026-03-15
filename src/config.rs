use serde::{Deserialize, Serialize};
use voxine::cam_controller::CameraConfig;

use crate::config_loader::{self, ConfigFile};

#[derive(Clone)]
pub struct LiveConfig {
    pub full_detail_distance: f32,
    pub full_detail_generation_distance: f32,
    pub render_distance: f32,
    pub max_chunks: usize,

    pub print_tps: bool,

    pub camera: CameraConfig,
    pub gpu_mesh_upload_time: f64,
}

impl config_loader::Live for LiveConfig {}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub full_detail_distance: f32,
    pub full_detail_generation_distance: f32,
    pub render_distance: f32,
    pub max_chunks: usize,

    pub print_tps: bool,

    pub camera: CameraConfig,
    pub gpu_mesh_upload_time: f64,

    pub starting_pos: [f32; 3],
    pub fov: f32,
    pub near_plane: f32,

    pub worker_count: usize,

    pub task_queue_cap: usize,

    pub discarded_tasks_queue_cap: usize,

    pub mesh_queue_cap: usize,
    pub chunk_queue_cap: usize,
    pub collider_queue_cap: usize,
    pub solid_map_queue_cap: usize,

    pub config_sender_cap: usize,
}

impl ConfigFile<LiveConfig> for Config {
    fn live(self) -> LiveConfig {
        LiveConfig {
            full_detail_distance: self.full_detail_distance,
            full_detail_generation_distance: self.full_detail_generation_distance,
            render_distance: self.render_distance,
            max_chunks: self.max_chunks,

            print_tps: self.print_tps,

            camera: self.camera,
            gpu_mesh_upload_time: self.gpu_mesh_upload_time,
        }
    }

    fn sender_cap(&self) -> usize {
        self.config_sender_cap
    }

    fn update(&mut self, update: LiveConfig) {
        self.full_detail_distance = update.full_detail_distance;
        self.render_distance = update.render_distance;
        self.max_chunks = update.max_chunks;
        self.print_tps = update.print_tps;
        self.gpu_mesh_upload_time = update.gpu_mesh_upload_time;
    }
}

impl Config {
    pub fn engine_config(&self) -> voxine::Config {
        voxine::Config {
            full_detail_distance: self.full_detail_generation_distance / 32.,
            total_generation_distance: self.render_distance / 32.,
            max_chunks: self.max_chunks,

            print_tps: self.print_tps,

            worker_count: self.worker_count,

            task_queue_cap: self.task_queue_cap,

            discarded_tasks_queue_cap: self.discarded_tasks_queue_cap,

            mesh_queue_cap: self.mesh_queue_cap,
            chunk_queue_cap: self.chunk_queue_cap,
            collider_queue_cap: self.collider_queue_cap,
            solid_map_queue_cap: self.solid_map_queue_cap,
        }
    }
}

impl LiveConfig {
    pub fn engine_config_update(&self) -> voxine::ConfigUpdate {
        voxine::ConfigUpdate {
            full_detail_distance: self.full_detail_generation_distance / 32.,
            total_generation_distance: self.render_distance / 32.,
            max_chunks: self.max_chunks,

            print_tps: self.print_tps,
        }
    }
}
