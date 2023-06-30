use block_mesh::{MergeVoxel, Voxel as MeshVoxel, VoxelVisibility};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
pub struct Voxel {
    id: u8,
}

impl Voxel {
    pub const EMPTY: Self = Self { id: 0 };
    pub const FILLED: Self = Self { id: 1 };
}

impl MeshVoxel for Voxel {
    fn get_visibility(&self) -> VoxelVisibility {
        if self.id == Voxel::FILLED.id {
            return VoxelVisibility::Opaque;
        }
        VoxelVisibility::Empty
    }
}

impl MergeVoxel for Voxel {
    type MergeValue = u8;

    fn merge_value(&self) -> Self::MergeValue {
        return self.id;
    }
}
