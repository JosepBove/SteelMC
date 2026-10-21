//! Enchanting table block behavior.

use std::sync::{Arc, Weak};

use glam::IVec3;
use steel_macros::block_behavior;
use steel_registry::{
    REGISTRY, TaggedRegistryExt,
    blocks::{BlockRef, block_state_ext::BlockStateExt},
    vanilla_block_entity_types,
    vanilla_block_tags::BlockTag,
};
use steel_utils::{BlockPos, BlockStateId, Downcast as _};

use crate::behavior::InventoryAccess;
use crate::behavior::block::{BlockBehavior, BlockEntityCreation};
use crate::behavior::context::{BlockHitResult, BlockPlaceContext, InteractionResult};
use crate::block_entity::{BLOCK_ENTITIES, entities::EnchantingTableBlockEntity};
use crate::entity::ai::path::PathComputationType;
use crate::inventory::menu::kinds::enchantment;
use crate::player::Player;
use crate::world::World;

/// Behavior for the enchanting table.
#[block_behavior]
pub struct EnchantingTableBlock {
    block: BlockRef,
}

/// Every offset two blocks out from the table, at table height and one above.
const fn bookshelf_offsets() -> [IVec3; 32] {
    let mut offsets = [IVec3::ZERO; 32];
    let mut count = 0;
    for y in 0..=1 {
        for x in -2..=2 {
            for z in -2..=2 {
                if x.abs() == 2 || z.abs() == 2 {
                    offsets[count] = IVec3::new(x, y, z);
                    count += 1;
                }
            }
        }
    }
    }
    offsets
}

impl EnchantingTableBlock {
    /// Bookshelf offsets around an enchanting table
    pub const BOOKSHELF_OFFSETS: [IVec3; 32] = bookshelf_offsets();

    /// Creates a new enchanting table block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }

    /// Checks whether the block at `offset` can provide enchanting power
    #[must_use]
    pub fn is_valid_book_shelf(world: &World, pos: BlockPos, offset: IVec3) -> bool {
        let provider = world
            .get_block_state(pos.offset(offset.x, offset.y, offset.z))
            .get_block();
        if !REGISTRY
            .blocks
            .is_in_tag(provider, &BlockTag::ENCHANTMENT_POWER_PROVIDER)
        {
            return false;
        }
        let transmitter = world
            .get_block_state(pos.offset(offset.x / 2, offset.y, offset.z / 2))
            .get_block();
        REGISTRY
            .blocks
            .is_in_tag(transmitter, &BlockTag::ENCHANTMENT_POWER_TRANSMITTER)
    }
}

impl BlockBehavior for EnchantingTableBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::from_registered_factory(BLOCK_ENTITIES.create(
            &vanilla_block_entity_types::ENCHANTING_TABLE,
            level,
            pos,
            state,
        ))
    }

    fn is_pathfindable(
        &self,
        _state: BlockStateId,
        _computation_type: PathComputationType,
    ) -> bool {
        false
    }

    /// Opens the enchanting table menu
    fn use_without_item(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        player: &Player,
        _hit_result: &BlockHitResult,
        _inv: &mut InventoryAccess,
    ) -> InteractionResult {
        let Some(block_entity) = world.get_block_entity(pos) else {
            return InteractionResult::Success;
        };
        let Some(table) = block_entity.downcast_ref::<EnchantingTableBlockEntity>() else {
            return InteractionResult::Success;
        };
        let title = table.display_name();
        let inventory = player.inventory.clone();
        let enchantment_seed = player.experience.lock().enchantment_seed();
        player.open_menu(title, move |context| {
            enchantment(
                inventory,
                context.container_id,
                pos,
                context.world,
                enchantment_seed,
            )
        });
        InteractionResult::Success
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use glam::DVec3;
    use steel_registry::{
        data_components::vanilla_components::CUSTOM_NAME, init_vanilla_registry,
        item_stack::ItemStack, vanilla_blocks, vanilla_items,
    };
    use steel_utils::{
        ChunkPos, Direction,
        types::{InteractionHand, UpdateFlags},
    };
    use text_components::TextComponent;

    use super::*;
    use crate::behavior::init_behaviors;
    use crate::behavior::items::BlockItem;
    use crate::behavior::{PlacementOrientation, PlacementSource};
    use crate::block_entity::init_block_entities;
    use crate::entity::Entity as _;
    use crate::test_support::{
        TestPlayerBuilder, TestWorld, fresh_test_world, insert_ready_full_chunk,
    };

    fn placed_table(key: &'static str) -> (TestWorld, BlockPos) {
        init_vanilla_registry();
        init_behaviors();
        init_block_entities();
        let world_fixture = fresh_test_world(key);
        let world = &world_fixture.world;
        let pos = BlockPos::new(8, 64, 8);
        insert_ready_full_chunk(world, ChunkPos::from_block_pos(pos));
        assert!(world.set_block(
            pos,
            vanilla_blocks::ENCHANTING_TABLE.default_state(),
            UpdateFlags::UPDATE_ALL,
        ));
        (world_fixture, pos)
    }

    #[test]

    #[test]
    fn bookshelves_count_only_with_a_clear_transmitter_block() {
        let (world_fixture, pos) = placed_table("enchanting_table_bookshelf_power");
        let world = &world_fixture.world;
        let offset = IVec3::new(2, 0, 0);
        assert!(!EnchantingTableBlock::is_valid_book_shelf(
            world, pos, offset
        ));

        assert!(world.set_block(
            pos.offset(2, 0, 0),
            vanilla_blocks::BOOKSHELF.default_state(),
            UpdateFlags::UPDATE_ALL,
        ));
        assert!(EnchantingTableBlock::is_valid_book_shelf(
            world, pos, offset
        ));

        assert!(world.set_block(
            pos.offset(1, 0, 0),
            vanilla_blocks::STONE.default_state(),
            UpdateFlags::UPDATE_ALL,
        ));
        assert!(!EnchantingTableBlock::is_valid_book_shelf(
            world, pos, offset
        ));
    }

    #[test]

}
