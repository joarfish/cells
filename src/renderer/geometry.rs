use super::utils::GpuVector3;

pub struct Geometry {
    pub vertices: Vec<GpuVector3>,
    pub normals: Vec<GpuVector3>,
    pub part_ids: Vec<u32>,
    pub indices: Vec<u16>
}

pub fn create_cube_geometry() -> Geometry {
    Geometry { // We need to split the vertices in three, because we want sharp edges
        vertices: vec![
            // Front
            GpuVector3::new( -0.5, -0.5, -0.5 ), // 0
            GpuVector3::new( 0.5, -0.5, -0.5 ), // 1
            GpuVector3::new( 0.5, 0.5, -0.5 ), // 2
            GpuVector3::new( -0.5, 0.5, -0.5 ), // 3
            // Back
            GpuVector3::new( -0.5, -0.5, 0.5 ), // 4
            GpuVector3::new( 0.5, -0.5, 0.5 ), // 5
            GpuVector3::new( 0.5, 0.5, 0.5 ), // 6
            GpuVector3::new( -0.5, 0.5, 0.5 ), // 7
            // Top
            GpuVector3::new( -0.5, 0.5, -0.5 ), // 8
            GpuVector3::new( 0.5, 0.5, -0.5 ), // 9
            GpuVector3::new( 0.5, 0.5, 0.5 ), // 10
            GpuVector3::new( -0.5, 0.5, 0.5 ), // 11
            // Bottom
            GpuVector3::new( -0.5, -0.5, -0.5 ), // 12
            GpuVector3::new( 0.5, -0.5, -0.5 ), // 13
            GpuVector3::new( 0.5, -0.5, 0.5 ), // 14
            GpuVector3::new( -0.5, -0.5, 0.5 ), // 15
            // Left
            GpuVector3::new( -0.5, -0.5, 0.5 ), // 16
            GpuVector3::new( -0.5, -0.5, -0.5 ), // 17
            GpuVector3::new( -0.5, 0.5, -0.5 ), // 18
            GpuVector3::new( -0.5, 0.5, 0.5 ), // 19
            // Right
            GpuVector3::new( 0.5, -0.5, 0.5 ), // 20
            GpuVector3::new( 0.5, -0.5, -0.5 ), // 21
            GpuVector3::new( 0.5, 0.5, -0.5 ), // 22
            GpuVector3::new( 0.5, 0.5, 0.5 ), // 23
        ],
        normals: vec![
            // Front
            GpuVector3::new( 0.0, 0.0, -1.0), // 0
            GpuVector3::new( 0.0, 0.0, -1.0), // 1
            GpuVector3::new( 0.0, 0.0, -1.0), // 2
            GpuVector3::new( 0.0, 0.0, -1.0), // 3
            // Back
            GpuVector3::new( 0.0, 0.0, 1.0), // 4
            GpuVector3::new( 0.0, 0.0, 1.0), // 5
            GpuVector3::new( 0.0, 0.0, 1.0), // 6
            GpuVector3::new( 0.0, 0.0, 1.0), // 7
            // Top
            GpuVector3::new( 0.0, 1.0, 0.0), // 8
            GpuVector3::new( 0.0, 1.0, 0.0), // 9
            GpuVector3::new( 0.0, 1.0, 0.0), // 10
            GpuVector3::new( 0.0, 1.0, 0.0), // 11
            // Bottom
            GpuVector3::new( 0.0, -1.0, 0.0), // 12
            GpuVector3::new( 0.0, -1.0, 0.0), // 13
            GpuVector3::new( 0.0, -1.0, 0.0), // 14
            GpuVector3::new( 0.0, -1.0, 0.0), // 15
            // Left
            GpuVector3::new( -1.0, 0.0, 0.0), // 16
            GpuVector3::new( -1.0, 0.0, 0.0), // 17
            GpuVector3::new( -1.0, 0.0, 0.0), // 18
            GpuVector3::new( -1.0, 0.0, 0.0), // 19
            // Right
            GpuVector3::new( 1.0, 0.0, 0.0), // 20
            GpuVector3::new( 1.0, 0.0, 0.0), // 21
            GpuVector3::new( 1.0, 0.0, 0.0), // 22
            GpuVector3::new( 1.0, 0.0, 0.0) // 23
        ],
        part_ids: vec![0; 24],
        indices: vec![
            // Front
            0, 2, 1,        0, 3, 2,
            // Back
            4, 5, 6,        4, 6, 7,
            // Top
            8, 10, 9,       8, 11, 10,
            // Bottom
            12, 13, 14,     12, 14, 15,
            // Left
            16, 18, 17,     16, 19, 18,
            // Right
            20, 21, 22,     20, 22, 23
        ]
    }
}