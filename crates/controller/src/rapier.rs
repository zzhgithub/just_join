//todo 物理引擎的接入

use bevy::prelude::{
    Commands, Entity, EventReader, Input, IntoSystemConfigs, IntoSystemSetConfigs, KeyCode, Plugin,
    PreUpdate, Query, Res, ResMut, SystemSet, Update, Vec3, With, Without,
};
use bevy_rapier3d::{
    prelude::{
        Collider, ColliderMassProperties, CollisionGroups, RapierColliderHandle,
        RapierConfiguration, RapierContext, RapierRigidBodyHandle, RigidBody,
    },
    rapier::prelude::{
        ColliderFlags, InteractionGroups, RigidBodyMassProps, RigidBodySet, RigidBodyVelocity,
    },
};

use crate::{
    controller::{
        controller_to_pitch, controller_to_yaw, BodyTag, CharacterController,
        CharacterControllerPlugin, ControllerSet, Mass,
    },
    events::ImpulseEvent,
};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum RapiserSet {
    BODY_TO_VELOCITY_SYSTEM,
    CONTROLLER_TO_RAPIER_DYNAMIC_IMPULSE_SYSTEM,
    CONTROLLER_TO_RAPIER_DYNAMIC_FORCE_SYSTEM,
    CREATE_MASS_FROM_RAPIER_SYSTEM,
    TOGGLE_FLY_MODE_SYSTEM,
}

pub struct RapierDynamicImpulseCharacterControllerPlugin;

impl Plugin for RapierDynamicImpulseCharacterControllerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(CharacterControllerPlugin)
            .configure_set(PreUpdate, RapiserSet::TOGGLE_FLY_MODE_SYSTEM)
            .configure_sets(
                Update,
                (
                    RapiserSet::CREATE_MASS_FROM_RAPIER_SYSTEM,
                    RapiserSet::BODY_TO_VELOCITY_SYSTEM,
                    RapiserSet::CONTROLLER_TO_RAPIER_DYNAMIC_IMPULSE_SYSTEM,
                )
                    .chain(),
            )
            .add_systems(
                PreUpdate,
                toggle_fly_mode
                    .in_set(RapiserSet::TOGGLE_FLY_MODE_SYSTEM)
                    .after(ControllerSet::INPUT_TO_EVENT),
            )
            .add_systems(
                Update,
                (
                    (create_mass_from_rapier).in_set(RapiserSet::CREATE_MASS_FROM_RAPIER_SYSTEM),
                    (body_to_velocity).in_set(RapiserSet::BODY_TO_VELOCITY_SYSTEM),
                    (controller_to_rapier_dynamic_impulse)
                        .in_set(RapiserSet::CONTROLLER_TO_RAPIER_DYNAMIC_IMPULSE_SYSTEM)
                        .after(RapiserSet::BODY_TO_VELOCITY_SYSTEM),
                    (controller_to_yaw, controller_to_pitch),
                ),
            );
    }
}

pub fn controller_to_rapier_dynamic_impulse(
    mut impulses: EventReader<ImpulseEvent>,
    mut context: ResMut<RapierContext>,
    mut query: Query<(&RapierRigidBodyHandle), With<BodyTag>>,
) {
    let mut impulse = Vec3::ZERO;
    for event in impulses.iter() {
        impulse += **event;
    }
    if impulse.length_squared() > 1E-6 {
        for ele in query.iter_mut() {
            let body = context.bodies.get_mut(ele.0);
            match body {
                Some(b) => {
                    b.apply_impulse(impulse.into(), true);
                }
                None => {}
            }
        }
    }
}

pub fn body_to_velocity(
    mut query: Query<(&RapierRigidBodyHandle, &mut CharacterController), With<BodyTag>>,
    mut context: ResMut<RapierContext>,
) {
    let context = &mut *context;
    for (velocity, mut controller) in query.iter_mut() {
        // controller.velocity = velocity.linvel.into();
        let body = context.bodies.get(velocity.0);
        match body {
            Some(b) => {
                controller.velocity = (*b.linvel()).into();
            }
            None => {}
        }
    }
}

pub fn create_mass_from_rapier(
    mut commands: Commands,
    query: Query<(Entity, &ColliderMassProperties), Without<Mass>>,
) {
    for (entity, mass_props) in query.iter() {
        let mut mass;
        // FIXME: need be testing
        match mass_props {
            ColliderMassProperties::Density(density) => mass = density.clone(),
            ColliderMassProperties::Mass(mass_data) => mass = mass_data.clone(),
            ColliderMassProperties::MassProperties(porp) => mass = porp.mass,
        }
        commands.entity(entity).insert(Mass::new(mass));
    }
}

const NO_GRAVITY: [f32; 3] = [0.0, 0.0, 0.0];
const GRAVITY: [f32; 3] = [0.0, -9.81, 0.0];

fn toggle_fly_mode(
    keyboard_input: Res<Input<KeyCode>>,
    mut rapier_config: ResMut<RapierConfiguration>,
    mut query: Query<(&CharacterController, &mut CollisionGroups)>,
) {
    for (controller, mut collider) in query.iter_mut() {
        if keyboard_input.just_pressed(controller.input_map.key_fly) {
            rapier_config.gravity = if controller.fly {
                // collider_flags.collision_groups = InteractionGroups::none();
                // fixme: 没有实现的功能 穿过物体的设置
                NO_GRAVITY.into()
            } else {
                // collider_flags.collision_groups = InteractionGroups::default();
                GRAVITY.into()
            };
        }
    }
}
