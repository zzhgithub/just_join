use block_mesh::{MergeVoxel, Voxel as MeshVoxel, VoxelVisibility};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
pub struct Voxel {
    pub id: u8,
}

impl Voxel {
    pub const VoxelIds: [u8; 4] = [1, 2, 3, 4];

    pub const EMPTY: Self = Self { id: 0 };
    pub const FILLED: Self = Self { id: 1 };
    // 土壤
    pub const soil: Self = Self { id: 2 };
    // 草坪
    pub const grass: Self = Self { id: 3 };
    // 岩石
    pub const stone: Self = Self { id: 1 };
}

impl MeshVoxel for Voxel {
    fn get_visibility(&self) -> VoxelVisibility {
        if self.id > 0 {
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
