use crate::{
    GgrsComponentSnapshot, GgrsSnapshots, LoadWorld, Rollback, RollbackFrameCount, SaveWorld,
};
use bevy::prelude::*;
use std::marker::PhantomData;

pub struct GgrsComponentSnapshotReflectPlugin<C>
where
    C: Component + Reflect + FromWorld,
{
    _phantom: PhantomData<C>,
}

impl<C> Default for GgrsComponentSnapshotReflectPlugin<C>
where
    C: Component + Reflect + FromWorld,
{
    fn default() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

impl<C> GgrsComponentSnapshotReflectPlugin<C>
where
    C: Component + Reflect + FromWorld,
{
    pub fn save(
        mut snapshots: ResMut<GgrsSnapshots<C, GgrsComponentSnapshot<C, Box<dyn Reflect>>>>,
        frame: Res<RollbackFrameCount>,
        query: Query<(&Rollback, &C)>,
    ) {
        let components = query
            .iter()
            .map(|(&rollback, component)| (rollback, component.as_reflect().clone_value()));
        let snapshot = GgrsComponentSnapshot::new(components);
        snapshots.push(frame.0, snapshot);
    }

    pub fn load(
        mut commands: Commands,
        mut snapshots: ResMut<GgrsSnapshots<C, GgrsComponentSnapshot<C, Box<dyn Reflect>>>>,
        frame: Res<RollbackFrameCount>,
        mut query: Query<(Entity, &Rollback, Option<&mut C>)>,
    ) {
        let snapshot = snapshots.rollback(frame.0).get();

        for (entity, rollback, component) in query.iter_mut() {
            let snapshot = snapshot.get(rollback);

            match (component, snapshot) {
                (Some(mut component), Some(snapshot)) => {
                    component.apply(snapshot.as_ref());
                }
                (Some(_), None) => {
                    commands.entity(entity).remove::<C>();
                }
                (None, Some(snapshot)) => {
                    let snapshot = snapshot.clone_value();

                    commands.add(move |world: &mut World| {
                        let mut component = C::from_world(world);
                        component.apply(snapshot.as_ref());
                        world.entity_mut(entity).insert(component);
                    })
                }
                (None, None) => {}
            }
        }
    }
}

impl<C> Plugin for GgrsComponentSnapshotReflectPlugin<C>
where
    C: Component + Reflect + FromWorld,
{
    fn build(&self, app: &mut App) {
        app.init_resource::<GgrsSnapshots<C, GgrsComponentSnapshot<C, Box<dyn Reflect>>>>()
            .add_systems(SaveWorld, Self::save)
            .add_systems(LoadWorld, Self::load);
    }
}
