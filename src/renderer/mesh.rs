use super::utils::GpuVector3;


pub struct Mesh {
    pub vertices: Vec<GpuVector3>,
    pub normals: Vec<GpuVector3>,
    pub indices: Vec<u16>
}

pub fn create_cube_mesh() -> Mesh {
    Mesh { // We need to split the vertices in three, because we want sharp edges
        vertices: vec![
            GpuVector3::new( -0.5, 0.5, -0.5 ),
            GpuVector3::new( 0.5, 0.5, -0.5 ),
            GpuVector3::new( -0.5, -0.5, -0.5 ),
            GpuVector3::new( 0.5, -0.5, -0.5 ),

            GpuVector3::new( -0.5, 0.5, 0.5 ),
            GpuVector3::new( 0.5, 0.5, 0.5 ),
            GpuVector3::new( -0.5, -0.5, 0.5 ),
            GpuVector3::new( 0.5, -0.5, 0.5 ),
        ],
        normals: vec![

        ],
        indices: vec![
            // Front
            0, 1, 2,    1, 3, 2,
            // Back
            4, 5, 6,    5, 7, 6,
            // Top
            0, 4, 5,    5, 1, 0,
            // Bottom
            2, 6, 7,    7, 3, 2,
            // Left
            4, 0, 6,    0, 6, 2,
            // Right
            1, 5, 3,    5, 7, 3
        ]
    }
}