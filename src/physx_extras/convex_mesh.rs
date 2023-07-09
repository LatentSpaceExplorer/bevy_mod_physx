use std::mem::MaybeUninit;

use physx::{
    math::{PxBounds3, PxVec3},
    traits::Class,
};

use physx_sys::{
    PxConvexMesh_getIndexBuffer,
    PxConvexMesh_getLocalBounds,
    PxConvexMesh_getMassInformation,
    PxConvexMesh_getNbPolygons,
    PxConvexMesh_getNbVertices,
    PxConvexMesh_getPolygonData,
    PxConvexMesh_getVertices,
    // TODO: SDF getters
    //PxConvexMesh_getSDF,
    //PxConvexMesh_getConcreteTypeName,
    PxConvexMesh_isGpuCompatible,
    //PxConvexMesh_release_mut,
    // TODO: high level wrapper for PxMassProperties
    PxMassProperties,
};

pub trait ConvexMeshExtras {
    fn get_nb_vertices(&self) -> u32;
    fn get_vertices(&self) -> &[PxVec3];
    fn get_index_buffer(&self) -> &[u8];
    fn get_nb_polygons(&self) -> u32;
    fn get_polygon_data(&self, index: u32) -> Option<HullPolygon>;
    fn get_mass_information(&self) -> PxMassProperties;
    fn get_local_bounds(&self) -> PxBounds3;
    fn is_gpu_compatible(&self) -> bool;
}

impl ConvexMeshExtras for physx::convex_mesh::ConvexMesh {
    /// Returns the number of vertices.
    fn get_nb_vertices(&self) -> u32 {
        unsafe { PxConvexMesh_getNbVertices(self.as_ptr()) }
    }

    /// Returns the vertices.
    fn get_vertices(&self) -> &[PxVec3] {
        let vertices = unsafe {
            std::slice::from_raw_parts(
                PxConvexMesh_getVertices(self.as_ptr()),
                self.get_nb_vertices() as usize,
            )
        };

        // SAFETY: PxVec3 is repr(transparent) wrapper of physx_sys::PxVec3
        unsafe { std::mem::transmute(vertices) }
    }

    /// Returns the index buffer.
    fn get_index_buffer(&self) -> &[u8] {
        let polygon_count = self.get_nb_polygons();

        // for each polygon index buffer contains its points,
        // so we take last polygon's index offset plus its length to calculate total size
        let index_buffer_length = if polygon_count > 0 {
            let last_polygon = self.get_polygon_data(polygon_count - 1).unwrap();
            last_polygon.index_base as usize + last_polygon.nb_verts as usize
        } else {
            0
        };

        unsafe {
            std::slice::from_raw_parts(
                PxConvexMesh_getIndexBuffer(self.as_ptr()),
                index_buffer_length,
            )
        }
    }

    /// Returns the number of polygons.
    fn get_nb_polygons(&self) -> u32 {
        unsafe { PxConvexMesh_getNbPolygons(self.as_ptr()) }
    }

    /// Returns the polygon data.
    fn get_polygon_data(&self, index: u32) -> Option<HullPolygon> {
        let mut polygon = MaybeUninit::uninit();

        if unsafe { PxConvexMesh_getPolygonData(self.as_ptr(), index, polygon.as_mut_ptr()) } {
            Some(unsafe { polygon.assume_init() }.into())
        } else {
            None
        }
    }

    /// Returns the mass properties of the mesh assuming unit density.
    fn get_mass_information(&self) -> PxMassProperties {
        let mut mass = MaybeUninit::uninit();
        let mut local_inertia = MaybeUninit::uninit();
        let mut local_center_of_mass = MaybeUninit::uninit();

        unsafe {
            PxConvexMesh_getMassInformation(
                self.as_ptr(),
                mass.as_mut_ptr(),
                local_inertia.as_mut_ptr(),
                local_center_of_mass.as_mut_ptr(),
            );

            PxMassProperties {
                inertiaTensor: local_inertia.assume_init(),
                centerOfMass: local_center_of_mass.assume_init(),
                mass: mass.assume_init(),
            }
        }
    }

    /// Returns the local-space (vertex space) AABB from the convex mesh.
    fn get_local_bounds(&self) -> PxBounds3 {
        unsafe { PxConvexMesh_getLocalBounds(self.as_ptr()) }.into()
    }

    /// This method decides whether a convex mesh is gpu compatible.
    fn is_gpu_compatible(&self) -> bool {
        unsafe { PxConvexMesh_isGpuCompatible(self.as_ptr()) }
    }
}


#[derive(Debug, Clone)]
pub struct HullPolygon {
    /// Plane equation for this polygon.
    pub plane: [f32; 4],
    /// Number of vertices/edges in the polygon.
    pub nb_verts: u16,
    /// Offset in index buffer.
    pub index_base: u16,
}

impl From<physx_sys::PxHullPolygon> for HullPolygon {
    fn from(value: physx_sys::PxHullPolygon) -> Self {
        Self {
            plane: value.mPlane,
            nb_verts: value.mNbVerts,
            index_base: value.mIndexBase,
        }
    }
}

impl From<HullPolygon> for physx_sys::PxHullPolygon {
    fn from(value: HullPolygon) -> Self {
        Self {
            mPlane: value.plane,
            mNbVerts: value.nb_verts,
            mIndexBase: value.index_base,
        }
    }
}