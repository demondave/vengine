use rapier3d::dynamics::{
    CCDSolver, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet, RigidBody,
    RigidBodyHandle, RigidBodySet,
};
use rapier3d::geometry::{BroadPhaseMultiSap, ColliderSet, DefaultBroadPhase, NarrowPhase};
use rapier3d::math::Vector;
use rapier3d::na::Vector3;
use rapier3d::pipeline::{PhysicsPipeline, QueryPipeline};
use rapier3d::prelude::{Collider, ColliderHandle};

pub struct Simulation {
    physics_pipeline: PhysicsPipeline,
    gravity: Vector<f32>,
    integration_parameters: IntegrationParameters,
    island_manager: IslandManager,
    broad_phase: BroadPhaseMultiSap,
    narrow_phase: NarrowPhase,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
}

impl Simulation {
    pub fn new(gravity: Vector3<f32>) -> Self {
        Simulation {
            physics_pipeline: PhysicsPipeline::new(),
            gravity,
            integration_parameters: IntegrationParameters::default(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
        }
    }

    pub fn step(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &(),
        );
    }

    pub fn add_rigid_body(&mut self, body: impl Into<RigidBody>) -> RigidBodyHandle {
        self.rigid_body_set.insert(body)
    }

    pub fn add_collider(
        &mut self,
        collider: impl Into<Collider>,
        parent_handle: Option<RigidBodyHandle>,
    ) -> ColliderHandle {
        if let Some(parent_handle) = parent_handle {
            self.collider_set
                .insert_with_parent(collider, parent_handle, &mut self.rigid_body_set)
        } else {
            self.collider_set.insert(collider)
        }
    }

    pub fn rigid_body_set(&self) -> &RigidBodySet {
        &self.rigid_body_set
    }
}
