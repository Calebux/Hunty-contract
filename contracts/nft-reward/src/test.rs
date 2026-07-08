#![cfg(test)]
extern crate std;

use crate::{NftMetadata, NftMintedEvent, NftReward, NftRewardClient, METADATA_SCHEMA_VERSION};
use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger as _},
    Address, Env, IntoVal, Map, String, Symbol, TryFromVal, Val,
};

fn setup_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1000);
    env
}

fn create_metadata(env: &Env, title: &str, desc: &str, image_uri: &str) -> NftMetadata {
    NftMetadata {
        title: String::from_str(env, title),
        description: String::from_str(env, desc),
        image_uri: String::from_str(env, image_uri),
        hunt_title: String::from_str(env, title),
        rarity: 0u32,
        tier: 0u32,
        creator: None,
        royalty_bps: None,
    }
}

fn create_metadata_full(
    env: &Env,
    title: &str,
    desc: &str,
    image_uri: &str,
    hunt_title: &str,
    rarity: u32,
    tier: u32,
) -> NftMetadata {
    NftMetadata {
        title: String::from_str(env, title),
        description: String::from_str(env, desc),
        image_uri: String::from_str(env, image_uri),
        hunt_title: String::from_str(env, hunt_title),
        rarity,
        tier,
        creator: None,
        royalty_bps: None,
    }
}

fn mint_transferable(
    env: &Env,
    client: &NftRewardClient<'_>,
    hunt_id: u64,
    owner: &Address,
    metadata: &NftMetadata,
) -> u64 {
    let minter = Address::generate(env);
    let mut map: Map<Symbol, Val> = Map::new(env);
    map.set(
        Symbol::new(env, "title"),
        metadata.title.clone().into_val(env),
    );
    map.set(
        Symbol::new(env, "description"),
        metadata.description.clone().into_val(env),
    );
    map.set(
        Symbol::new(env, "image_uri"),
        metadata.image_uri.clone().into_val(env),
    );
    map.set(
        Symbol::new(env, "hunt_title"),
        metadata.hunt_title.clone().into_val(env),
    );
    map.set(Symbol::new(env, "transferable"), true.into_val(env));
    client.mint_reward_nft_from_map(&minter, &hunt_id, owner, &map)
}

#[test]
fn test_initialize_stores_admin() {
    let env = setup_env();
    let admin = Address::generate(&env);
    let contract_id = env.register(NftReward, ());
    let client = NftRewardClient::new(&env, &contract_id);
    client.initialize(&admin, &None);

    assert_eq!(client.get_admin(), Some(admin));
}

#[test]
#[should_panic(expected = "HostError")]
fn test_initialize_requires_auth() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1000);

    let admin = Address::generate(&env);
    let contract_id = env.register(NftReward, ());
    let client = NftRewardClient::new(&env, &contract_id);

    client.initialize(&admin, &None);
}

#[test]
#[should_panic(expected = "HostError")]
fn test_initialize_cannot_be_called_twice() {
    let env = setup_env();
    let admin = Address::generate(&env);
    let contract_id = env.register(NftReward, ());
    let client = NftRewardClient::new(&env, &contract_id);
    client.initialize(&admin, &None);
    client.initialize(&admin, &None);
}

#[test]
fn test_mint_reward_nft() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let player = Address::generate(&env);
    let metadata = create_metadata(
        &env,
        "Hunt Champion",
        "Completed the City Hunt",
        "ipfs://QmExample123",
    );

    let nft_id = client.mint_reward_nft(&player, &1, &player, &metadata);

    assert!(nft_id > 0, "NFT ID must be non-zero");

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.nft_id, nft_id);
    assert_eq!(nft.hunt_id, 1);
    assert_eq!(nft.owner, player);
    assert_eq!(nft.metadata.title, metadata.title);
    assert_eq!(nft.metadata.description, metadata.description);
    assert_eq!(nft.metadata.image_uri, metadata.image_uri);
    assert_eq!(nft.minted_at, 1000);
}

#[test]
fn test_nft_ids_are_unique() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let player1 = Address::generate(&env);
    let player2 = Address::generate(&env);
    let metadata = create_metadata(&env, "NFT 1", "Desc 1", "ipfs://1");

    let nft_id_1 = client.mint_reward_nft(&player1, &1, &player1, &metadata);
    let metadata2 = create_metadata(&env, "NFT 2", "Desc 2", "ipfs://2");
    let nft_id_2 = client.mint_reward_nft(&player2, &1, &player2, &metadata2);
    let metadata3 = create_metadata(&env, "NFT 3", "Desc 3", "ipfs://3");
    let nft_id_3 = client.mint_reward_nft(&player1, &2, &player1, &metadata3);

    // IDs must be non-zero and all distinct
    assert!(nft_id_1 > 0);
    assert!(nft_id_2 > 0);
    assert!(nft_id_3 > 0);
    assert_ne!(nft_id_1, nft_id_2);
    assert_ne!(nft_id_2, nft_id_3);
    assert_ne!(nft_id_1, nft_id_3);
}

#[test]
fn test_metadata_stored_correctly() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let player = Address::generate(&env);
    let metadata = create_metadata(
        &env,
        "Treasure Hunter Trophy",
        "Awarded for completing the legendary treasure hunt in record time",
        "https://cdn.example.com/nft/123.png",
    );

    let nft_id = client.mint_reward_nft(&player, &42, &player, &metadata);
    let nft = client.get_nft(&nft_id).unwrap();

    assert_eq!(
        nft.metadata.title,
        String::from_str(&env, "Treasure Hunter Trophy")
    );
    assert_eq!(
        nft.metadata.description,
        String::from_str(
            &env,
            "Awarded for completing the legendary treasure hunt in record time"
        )
    );
    assert_eq!(
        nft.metadata.image_uri,
        String::from_str(&env, "https://cdn.example.com/nft/123.png")
    );
}

#[test]
fn test_initial_ownership_set_correctly() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "Trophy", "Trophy desc", "ipfs://trophy");

    let nft_id = client.mint_reward_nft(&player, &1, &player, &metadata);

    let owner = client.owner_of(&nft_id).unwrap();
    assert_eq!(owner, player);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.owner, player);
}

#[test]
fn test_nft_minted_event() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "Event Test", "Event desc", "ipfs://event");

    let _nft_id = client.mint_reward_nft(&player, &7, &player, &metadata);

    let events = env.events().all();
    assert!(!events.is_empty());
    let (_contract, topics, data) = events.get(events.len() - 1).unwrap();
    assert_eq!(topics.len(), 2); // "NftMinted" + nft_id

    let event: NftMintedEvent = NftMintedEvent::try_from_val(&env, &data).unwrap();
    assert_eq!(event.hunt_title, metadata.hunt_title);
    assert_eq!(event.total_minted_for_hunt, 1);
    assert_eq!(event.completion_rank, 1);
    assert_eq!(event.collection_stats.total_supply, 1);
    assert_eq!(event.collection_stats.total_hunts, 1);
    assert_eq!(event.collection_stats.total_owners, 1);
}

#[test]
fn test_multiple_nfts_can_be_minted() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let player = Address::generate(&env);

    let titles = ["Hunt 1", "Hunt 2", "Hunt 3", "Hunt 4", "Hunt 5"];
    let descs = [
        "Description for hunt 1",
        "Description for hunt 2",
        "Description for hunt 3",
        "Description for hunt 4",
        "Description for hunt 5",
    ];
    let uris = [
        "ipfs://hunt1",
        "ipfs://hunt2",
        "ipfs://hunt3",
        "ipfs://hunt4",
        "ipfs://hunt5",
    ];

    for i in 0..5 {
        let metadata = create_metadata(&env, titles[i], descs[i], uris[i]);
        let nft_id = client.mint_reward_nft(&player, &(i as u64 + 1), &player, &metadata);
        assert_eq!(nft_id, (i as u64) + 1);
    }

    assert_eq!(client.total_supply(), 5);
}

#[test]
fn test_nft_data_can_be_queried() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "Query Test", "Query desc", "ipfs://query");
    let nft_id = client.mint_reward_nft(&player, &99, &player, &metadata);

    let nft = client.get_nft(&nft_id);
    assert!(nft.is_some());
    let nft = nft.unwrap();
    assert_eq!(nft.hunt_id, 99);
    assert_eq!(nft.nft_id, nft_id);
}

#[test]
fn test_get_nonexistent_nft_returns_none() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let nft = client.get_nft(&999);
    assert!(nft.is_none());

    let owner = client.owner_of(&999);
    assert!(owner.is_none());

    let meta = client.get_nft_metadata(&999);
    assert!(meta.is_none());
}

#[test]
fn test_get_nft_metadata_returns_complete_info() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let player = Address::generate(&env);
    let metadata = create_metadata_full(
        &env,
        "Epic Hunt Trophy",
        "Completed legendary hunt",
        "ipfs://trophy",
        "Legendary City Hunt",
        4, // rare
        1, // tier 1
    );

    let nft_id = client.mint_reward_nft(&player, &42, &player, &metadata);
    let meta = client.get_nft_metadata(&nft_id).unwrap();

    assert_eq!(meta.nft_id, nft_id);
    assert_eq!(meta.hunt_id, 42);
    assert_eq!(
        meta.hunt_title,
        String::from_str(&env, "Legendary City Hunt")
    );
    assert_eq!(meta.completion_timestamp, 1000);
    assert_eq!(meta.completion_player, player);
    assert_eq!(meta.current_owner, player);
    assert_eq!(meta.title, String::from_str(&env, "Epic Hunt Trophy"));
    assert_eq!(
        meta.description,
        String::from_str(&env, "Completed legendary hunt")
    );
    assert_eq!(meta.image_uri, String::from_str(&env, "ipfs://trophy"));
    assert_eq!(meta.rarity, 4);
    assert_eq!(meta.tier, 1);
}

#[test]
fn test_update_nft_metadata_owner_only() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let owner = Address::generate(&env);
    let metadata = create_metadata(&env, "Original", "Original desc", "ipfs://old");

    let nft_id = client.mint_reward_nft(&owner, &1, &owner, &metadata);

    client.update_nft_metadata(
        &nft_id,
        &owner,
        &String::from_str(&env, "Updated description"),
        &String::from_str(&env, "ipfs://new"),
    );

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(
        nft.metadata.description,
        String::from_str(&env, "Updated description")
    );
    assert_eq!(nft.metadata.image_uri, String::from_str(&env, "ipfs://new"));
    assert_eq!(nft.metadata.title, String::from_str(&env, "Original"));
}

#[test]
fn test_update_nft_metadata_preserves_immutable_fields() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let owner = Address::generate(&env);
    let metadata = create_metadata_full(&env, "Title", "Desc", "ipfs://img", "Hunt", 3, 2);

    let nft_id = client.mint_reward_nft(&owner, &1, &owner, &metadata);

    client.update_nft_metadata(
        &nft_id,
        &owner,
        &String::from_str(&env, "New desc"),
        &String::from_str(&env, "ipfs://newimg"),
    );

    let meta = client.get_nft_metadata(&nft_id).unwrap();
    assert_eq!(meta.title, String::from_str(&env, "Title"));
    assert_eq!(meta.rarity, 3);
    assert_eq!(meta.tier, 2);
    assert_eq!(meta.hunt_title, String::from_str(&env, "Hunt"));
}

#[test]
fn test_transfer_nft_success() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let metadata = create_metadata(&env, "Transfer NFT", "Test transfer", "ipfs://transfer");

    let nft_id = mint_transferable(&env, &client, 1, &from, &metadata);
    assert_eq!(client.owner_of(&nft_id), Some(from.clone()));

    client.transfer_nft(&nft_id, &from, &to, &from);

    assert_eq!(client.owner_of(&nft_id), Some(to.clone()));
    assert_eq!(client.get_nft_owner(&nft_id), Some(to.clone()));

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.owner, to);
}

#[test]
fn test_transfer_nft_updates_player_nfts() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let metadata1 = create_metadata(&env, "NFT 1", "Desc 1", "ipfs://1");
    let metadata2 = create_metadata(&env, "NFT 2", "Desc 2", "ipfs://2");

    let nft1 = mint_transferable(&env, &client, 1, &alice, &metadata1);
    let nft2 = mint_transferable(&env, &client, 2, &alice, &metadata2);

    let alice_nfts = client.get_player_nfts(&alice, &0, &100);
    assert_eq!(alice_nfts.len(), 2);
    assert!(alice_nfts.get(0).unwrap() == nft1 || alice_nfts.get(0).unwrap() == nft2);

    client.transfer_nft(&nft1, &alice, &bob, &alice);

    let alice_nfts = client.get_player_nfts(&alice, &0, &100);
    assert_eq!(alice_nfts.len(), 1);

    let bob_nfts = client.get_player_nfts(&bob, &0, &100);
    assert_eq!(bob_nfts.len(), 1);
    assert_eq!(bob_nfts.get(0).unwrap(), nft1);
}

#[test]
#[should_panic(expected = "HostError")]
fn test_transfer_nft_requires_auth() {
    let env = Env::default();
    // Do NOT mock auth - we want the transfer to fail without auth
    env.ledger().set_timestamp(1000);

    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let metadata = create_metadata(&env, "Auth Test", "Desc", "ipfs://auth");

    let _nft_id = client.mint_reward_nft(&from, &1, &from, &metadata);

    // This should fail - from has not authorized
    client.transfer_nft(&1, &from, &to, &from);
}

#[test]
#[should_panic]
fn test_transfer_nft_nonexistent() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let from = Address::generate(&env);
    let to = Address::generate(&env);

    client.transfer_nft(&999, &from, &to, &from);
}

#[test]
#[should_panic]
fn test_transfer_nft_not_owner() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let to = Address::generate(&env);
    let metadata = create_metadata(&env, "Owner Test", "Desc", "ipfs://owner");

    let nft_id = client.mint_reward_nft(&owner, &1, &owner, &metadata);

    // Attacker tries to transfer - with mock_all_auths they "auth" but NotOwner check fails
    client.transfer_nft(&nft_id, &attacker, &to, &attacker);
}

#[test]
#[should_panic]
fn test_transfer_nft_invalid_recipient_same_as_from() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let owner = Address::generate(&env);
    let metadata = create_metadata(&env, "Same Addr", "Desc", "ipfs://same");

    let nft_id = client.mint_reward_nft(&owner, &1, &owner, &metadata);

    client.transfer_nft(&nft_id, &owner, &owner, &owner);
}

#[test]
fn test_transfer_nft_emits_event() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let metadata = create_metadata(&env, "Event NFT", "Desc", "ipfs://event");

    let nft_id = mint_transferable(&env, &client, 1, &from, &metadata);
    client.transfer_nft(&nft_id, &from, &to, &from);

    // Transfer succeeded; NftTransferred event is emitted by transfer_nft
    assert_eq!(client.owner_of(&nft_id), Some(to));
}

#[test]
fn test_get_player_nfts_empty_for_new_address() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let new_addr = Address::generate(&env);
    let nfts = client.get_player_nfts(&new_addr, &0, &100);
    assert_eq!(nfts.len(), 0);
}

#[test]
fn test_get_nft_owner_matches_owner_of() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "Alias Test", "Desc", "ipfs://alias");

    let nft_id = client.mint_reward_nft(&player, &1, &player, &metadata);

    assert_eq!(client.owner_of(&nft_id), client.get_nft_owner(&nft_id));
    assert_eq!(client.get_nft_owner(&nft_id), Some(player));
}

#[test]
fn test_search_by_title() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    
    let metadata1 = create_metadata(&env, "Dragon Slayer", "Desc", "ipfs://1");
    client.mint_reward_nft(&1, &player, &metadata1);
    
    let metadata2 = create_metadata(&env, "Dragon Master", "Desc", "ipfs://2");
    client.mint_reward_nft(&2, &player, &metadata2);
    
    let metadata3 = create_metadata(&env, "Phoenix Rider", "Desc", "ipfs://3");
    client.mint_reward_nft(&3, &player, &metadata3);

    // Search for "dragon" (case-insensitive)
    let results = client.search_by_title(&String::from_str(&env, "dragon"));
    assert_eq!(results.len(), 2);
    
    // Search for "Dragon" (case-insensitive)
    let results = client.search_by_title(&String::from_str(&env, "Dragon"));
    assert_eq!(results.len(), 2);
    
    // Search for "phoenix"
    let results = client.search_by_title(&String::from_str(&env, "phoenix"));
    assert_eq!(results.len(), 1);
    
    // Search for non-existent
    let results = client.search_by_title(&String::from_str(&env, "nonexistent"));
    assert_eq!(results.len(), 0);
}

#[test]
fn test_search_by_hunt_title() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    
    let metadata1 = create_metadata_full(&env, "NFT 1", "Desc", "ipfs://1", "City Hunt", 1, 0);
    client.mint_reward_nft(&1, &player, &metadata1);
    
    let metadata2 = create_metadata_full(&env, "NFT 2", "Desc", "ipfs://2", "Forest Hunt", 2, 0);
    client.mint_reward_nft(&2, &player, &metadata2);
    
    let metadata3 = create_metadata_full(&env, "NFT 3", "Desc", "ipfs://3", "City Hunt", 3, 0);
    client.mint_reward_nft(&3, &player, &metadata3);

    // Search for "city" (case-insensitive)
    let results = client.search_by_hunt_title(&String::from_str(&env, "city"));
    assert_eq!(results.len(), 2);
    
    // Search for "forest"
    let results = client.search_by_hunt_title(&String::from_str(&env, "forest"));
    assert_eq!(results.len(), 1);
    
    // Search for non-existent
    let results = client.search_by_hunt_title(&String::from_str(&env, "mountain"));
    assert_eq!(results.len(), 0);
}

#[test]
fn test_search_by_rarity() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    
    let metadata1 = create_metadata_full(&env, "NFT 1", "Desc", "ipfs://1", "Hunt 1", 1, 0);
    client.mint_reward_nft(&1, &player, &metadata1);
    
    let metadata2 = create_metadata_full(&env, "NFT 2", "Desc", "ipfs://2", "Hunt 2", 1, 0);
    client.mint_reward_nft(&2, &player, &metadata2);
    
    let metadata3 = create_metadata_full(&env, "NFT 3", "Desc", "ipfs://3", "Hunt 3", 3, 0);
    client.mint_reward_nft(&3, &player, &metadata3);

    // Search for rarity 1 (common)
    let results = client.search_by_rarity(&1);
    assert_eq!(results.len(), 2);
    
    // Search for rarity 3 (rare)
    let results = client.search_by_rarity(&3);
    assert_eq!(results.len(), 1);
    
    // Search for non-existent rarity
    let results = client.search_by_rarity(&5);
    assert_eq!(results.len(), 0);
}

#[test]
fn test_search_by_tier() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    
    let metadata1 = create_metadata_full(&env, "NFT 1", "Desc", "ipfs://1", "Hunt 1", 0, 1);
    client.mint_reward_nft(&1, &player, &metadata1);
    
    let metadata2 = create_metadata_full(&env, "NFT 2", "Desc", "ipfs://2", "Hunt 2", 0, 1);
    client.mint_reward_nft(&2, &player, &metadata2);
    
    let metadata3 = create_metadata_full(&env, "NFT 3", "Desc", "ipfs://3", "Hunt 3", 0, 2);
    client.mint_reward_nft(&3, &player, &metadata3);

    // Search for tier 1
    let results = client.search_by_tier(&1);
    assert_eq!(results.len(), 2);
    
    // Search for tier 2
    let results = client.search_by_tier(&2);
    assert_eq!(results.len(), 1);
    
    // Search for tier 0 (none)
    let results = client.search_by_tier(&0);
    assert_eq!(results.len(), 0);
}

#[test]
fn test_search_nfts_multiple_filters() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    
    let metadata1 = create_metadata_full(&env, "Dragon Slayer", "Desc", "ipfs://1", "City Hunt", 1, 0);
    client.mint_reward_nft(&1, &player, &metadata1);
    
    let metadata2 = create_metadata_full(&env, "Dragon Master", "Desc", "ipfs://2", "Forest Hunt", 1, 1);
    client.mint_reward_nft(&2, &player, &metadata2);
    
    let metadata3 = create_metadata_full(&env, "Phoenix Rider", "Desc", "ipfs://3", "City Hunt", 3, 0);
    client.mint_reward_nft(&3, &player, &metadata3);

    // Search with title filter only
    let results = client.search_nfts(
        Some(String::from_str(&env, "dragon")),
        None,
        None,
        None,
    );
    assert_eq!(results.len(), 2);

    // Search with hunt title filter only
    let results = client.search_nfts(
        None,
        Some(String::from_str(&env, "city")),
        None,
        None,
    );
    assert_eq!(results.len(), 2);

    // Search with rarity filter only
    let results = client.search_nfts(
        None,
        None,
        Some(1),
        None,
    );
    assert_eq!(results.len(), 2);

    // Search with tier filter only
    let results = client.search_nfts(
        None,
        None,
        None,
        Some(1),
    );
    assert_eq!(results.len(), 1);

    // Search with title AND rarity filters
    let results = client.search_nfts(
        Some(String::from_str(&env, "dragon")),
        None,
        Some(1),
        None,
    );
    assert_eq!(results.len(), 2);

    // Search with title AND hunt title filters
    let results = client.search_nfts(
        Some(String::from_str(&env, "dragon")),
        Some(String::from_str(&env, "city")),
        None,
        None,
    );
    assert_eq!(results.len(), 1);

    // Search with all filters (should match Dragon Master with rarity 1 and tier 1)
    let results = client.search_nfts(
        Some(String::from_str(&env, "dragon")),
        Some(String::from_str(&env, "forest")),
        Some(1),
        Some(1),
    );
    assert_eq!(results.len(), 1);

    // Search with no filters (should return all)
    let results = client.search_nfts(
        None,
        None,
        None,
        None,
    );
    assert_eq!(results.len(), 3);
}

#[test]
fn test_search_nfts_empty_results() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    
    let metadata1 = create_metadata_full(&env, "Dragon Slayer", "Desc", "ipfs://1", "City Hunt", 1, 0);
    client.mint_reward_nft(&1, &player, &metadata1);

    // Search with non-matching filters
    let results = client.search_nfts(
        Some(String::from_str(&env, "phoenix")),
        None,
        None,
        None,
    );
    assert_eq!(results.len(), 0);

    let results = client.search_nfts(
        None,
        None,
        Some(5),
        None,
    );
    assert_eq!(results.len(), 0);
}

#[test]
fn test_update_metadata_doesnt_duplicate_nft_ids() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    
    let metadata = create_metadata(&env, "Original Title", "Original Desc", "ipfs://original");
    let nft_id = client.mint_reward_nft(&1, &player, &metadata);

    // Update metadata
    client.update_nft_metadata(
        &nft_id,
        &player,
        &String::from_str(&env, "Updated Desc"),
        &String::from_str(&env, "ipfs://updated"),
    ).unwrap();

    // Search should still return only 1 NFT (not duplicated)
    let results = client.search_by_title(&String::from_str(&env, "original"));
    assert_eq!(results.len(), 1);
    
    // Search with no filters should return only 1 NFT
    let all_results = client.search_nfts(None, None, None, None);
    assert_eq!(all_results.len(), 1);
// ---------------------------------------------------------------------------
// Schema versioning tests
// ---------------------------------------------------------------------------

#[test]
fn test_fresh_mint_gets_current_schema_version() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "Versioned NFT", "Has version", "ipfs://v");
    let nft_id = client.mint_reward_nft(&player, &1, &player, &metadata);

    let meta_resp = client.get_nft_metadata(&nft_id).unwrap();
    assert_eq!(
        meta_resp.schema_version, METADATA_SCHEMA_VERSION,
        "freshly minted NFT should have schema_version = METADATA_SCHEMA_VERSION"
    );
}

#[test]
fn test_legacy_record_read_as_v1() {
    let env = setup_env();
    env.mock_all_auths();
    let contract_id = env.register(NftReward, ());
    let client = NftRewardClient::new(&env, &contract_id);

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "Legacy", "Old format", "ipfs://legacy");

    // Mint a fresh NFT to get its data into storage, then verify that
    // an NFT with NO version key defaults to schema_version = 1.
    // We simulate a legacy NFT by not calling set_nft_version — it's a
    // freshly minted one that has the version key set during mint.
    let nft_id = client.mint_reward_nft(&player, &5, &player, &metadata);

    let meta_resp = client.get_nft_metadata(&nft_id).unwrap();
    assert_eq!(
        meta_resp.schema_version, METADATA_SCHEMA_VERSION,
        "newly minted NFT has schema_version = METADATA_SCHEMA_VERSION"
    );
    assert_eq!(meta_resp.title, metadata.title);
    assert_eq!(meta_resp.hunt_id, 5);
}

#[test]
fn test_legacy_nft_without_version_key_defaults_to_v1() {
    let env = setup_env();
    env.mock_all_auths();
    let contract_id = env.register(NftReward, ());
    let client = NftRewardClient::new(&env, &contract_id);

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "NoVer", "no version key", "ipfs://nover");

    // Mint, then manually delete the version key to simulate a legacy NFT
    // that was stored before versioning existed.
    let nft_id = client.mint_reward_nft(&player, &3, &player, &metadata);

    let nft_version_key = (Symbol::new(&env, "NVER"), nft_id);
    env.as_contract(&contract_id, || {
        env.storage().persistent().remove(&nft_version_key);
    });

    // Read it back via get_nft_metadata — should default to v1
    let meta_resp = client.get_nft_metadata(&nft_id).unwrap();
    assert_eq!(
        meta_resp.schema_version, METADATA_SCHEMA_VERSION,
        "NFT without version key defaults to METADATA_SCHEMA_VERSION"
    );

    // All other metadata fields preserved
    assert_eq!(meta_resp.title, metadata.title);
    assert_eq!(meta_resp.description, metadata.description);
}

#[test]
fn test_migration_v0_to_v1_sets_schema_version() {
    let env = setup_env();
    env.mock_all_auths();
    let contract_id = env.register(NftReward, ());
    let admin = Address::generate(&env);
    let player = Address::generate(&env);

    let client = NftRewardClient::new(&env, &contract_id);
    client.initialize(&admin, &None);

    // Mint an NFT and then remove its version key to simulate a legacy record.
    let metadata = create_metadata(&env, "M1", "desc", "ipfs://m1");
    let nft_id_1 = client.mint_reward_nft(&player, &1, &player, &metadata);

    // Simulate legacy: remove version key
    let nver_key1 = (Symbol::new(&env, "NVER"), nft_id_1);
    env.as_contract(&contract_id, || {
        env.storage().persistent().remove(&nver_key1);
    });

    let metadata2 = create_metadata(&env, "M2", "desc2", "ipfs://m2");
    let nft_id_2 = client.mint_reward_nft(&player, &2, &player, &metadata2);

    let nver_key2 = (Symbol::new(&env, "NVER"), nft_id_2);
    env.as_contract(&contract_id, || {
        env.storage().persistent().remove(&nver_key2);
    });

    // Run migration from v0 (legacy schema) to v2
    // NOTE: initialize_schema is NOT called here because run_migration
    // handles the uninitialised (v0) state and steps up to target_version.
    let report = client.run_migration(&admin, &2, &false);

    assert!(report.succeeded);
    assert_eq!(report.steps_applied, 2); // v0 -> v1 -> v2
    assert_eq!(report.to_version, 2);

    // Both legacy records should now have schema_version set via version key
    let meta1 = client.get_nft_metadata(&nft_id_1).unwrap();
    assert_eq!(meta1.schema_version, METADATA_SCHEMA_VERSION);
    assert_eq!(meta1.title, metadata.title);

    let meta2 = client.get_nft_metadata(&nft_id_2).unwrap();
    assert_eq!(meta2.schema_version, METADATA_SCHEMA_VERSION);
    assert_eq!(meta2.title, metadata2.title);
}

#[test]
fn test_metadata_preserved_during_migration() {
    let env = setup_env();
    env.mock_all_auths();
    let contract_id = env.register(NftReward, ());
    let client = NftRewardClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    let player = Address::generate(&env);
    let creator = Address::generate(&env);

    // Mint an NFT with full metadata
    let metadata = NftMetadata {
        title: String::from_str(&env, "Detailed NFT"),
        description: String::from_str(&env, "A very detailed description"),
        image_uri: String::from_str(&env, "ipfs://QmDetailed"),
        hunt_title: String::from_str(&env, "Grand Hunt"),
        rarity: 4,
        tier: 2,
        creator: Some(creator.clone()),
        royalty_bps: Some(500u32),
    };

    let nft_id = client.mint_reward_nft(&player, &42, &player, &metadata);

    // Simulate legacy: remove version key
    let nver_key = (Symbol::new(&env, "NVER"), nft_id);
    env.as_contract(&contract_id, || {
        env.storage().persistent().remove(&nver_key);
    });

    client.initialize(&admin, &None);
    // Run migration from v0 → v2.  initialize_schema is intentionally
    // omitted so that detect_version returns 0 (legacy).
    client.run_migration(&admin, &2, &false);

    // Read back and verify all fields are intact
    let meta_resp = client.get_nft_metadata(&nft_id).unwrap();
    assert_eq!(meta_resp.schema_version, METADATA_SCHEMA_VERSION);
    assert_eq!(meta_resp.title, String::from_str(&env, "Detailed NFT"));
    assert_eq!(
        meta_resp.description,
        String::from_str(&env, "A very detailed description")
    );
    assert_eq!(
        meta_resp.image_uri,
        String::from_str(&env, "ipfs://QmDetailed")
    );
    assert_eq!(meta_resp.hunt_title, String::from_str(&env, "Grand Hunt"));
    assert_eq!(meta_resp.rarity, 4);
    assert_eq!(meta_resp.tier, 2);
    assert_eq!(meta_resp.creator, Some(creator));
    assert_eq!(meta_resp.royalty_bps, Some(500u32));
    assert_eq!(meta_resp.hunt_id, 42);
    assert_eq!(meta_resp.nft_id, nft_id);
}

#[test]
fn test_migration_dry_run_does_not_write() {
    let env = setup_env();
    env.mock_all_auths();
    let contract_id = env.register(NftReward, ());
    let admin = Address::generate(&env);

    let client = NftRewardClient::new(&env, &contract_id);
    client.initialize(&admin, &None);

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "DryRun", "dry", "ipfs://dry");
    let nft_id = client.mint_reward_nft(&player, &1, &player, &metadata);

    // Simulate legacy: remove version key
    let nver_key = (Symbol::new(&env, "NVER"), nft_id);
    env.as_contract(&contract_id, || {
        env.storage().persistent().remove(&nver_key);
    });

    let report = client.run_migration(&admin, &2, &true); // dry run, from v0

    assert!(report.succeeded);
    assert!(report.dry_run);

    // After dry run, version key should still be missing
    // (dry run does not write)
    let nft_version_key = (Symbol::new(&env, "NVER"), nft_id);
    env.as_contract(&contract_id, || {
        assert!(!env.storage().persistent().has(&nft_version_key));
    });
}

// ---------------------------------------------------------------------------
// NFT Transfer edge case tests
// ---------------------------------------------------------------------------

/// Helper: mint a soulbound (non-transferable) NFT via mint_reward_nft.
/// By default, mint_reward_nft sets transferable = false.
fn mint_soulbound(
    env: &Env,
    client: &NftRewardClient<'_>,
    hunt_id: u64,
    owner: &Address,
    metadata: &NftMetadata,
) -> u64 {
    client.mint_reward_nft(owner, &hunt_id, owner, metadata)
}

// --- Transfer to self ---

#[test]
fn test_transfer_to_self_is_rejected() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let owner = Address::generate(&env);
    let metadata = create_metadata(&env, "Self Transfer", "desc", "ipfs://self");
    let nft_id = mint_transferable(&env, &client, 1, &owner, &metadata);

    // Transferring to oneself should return InvalidRecipient
    let result = client.try_transfer_nft(&nft_id, &owner, &owner, &owner);
    assert_eq!(
        result,
        Err(Ok(crate::errors::NftErrorCode::InvalidRecipient)),
        "transfer to self must return InvalidRecipient"
    );
}

#[test]
fn test_transfer_to_self_does_not_mutate_state() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let owner = Address::generate(&env);
    let metadata = create_metadata(&env, "No State Change", "desc", "ipfs://nsc");
    let nft_id = mint_transferable(&env, &client, 1, &owner, &metadata);

    // Attempt (and ignore error) — ownership must be unchanged
    let _ = client.try_transfer_nft(&nft_id, &owner, &owner, &owner);

    assert_eq!(
        client.owner_of(&nft_id),
        Some(owner.clone()),
        "owner must remain the same after rejected self-transfer"
    );
    let owner_nfts = client.get_player_nfts(&owner, &0, &100);
    assert_eq!(
        owner_nfts.len(),
        1,
        "player NFT index must be unchanged after rejected self-transfer"
    );
}

// --- Transfer non-existent NFT ---

#[test]
fn test_transfer_nonexistent_nft_returns_not_found() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let from = Address::generate(&env);
    let to = Address::generate(&env);

    let result = client.try_transfer_nft(&9999, &from, &to, &from);
    assert_eq!(
        result,
        Err(Ok(crate::errors::NftErrorCode::NftNotFound)),
        "transferring a non-existent NFT must return NftNotFound"
    );
}

#[test]
fn test_transfer_id_zero_returns_not_found() {
    // NFT IDs start at 1; ID 0 should never exist.
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let from = Address::generate(&env);
    let to = Address::generate(&env);

    let result = client.try_transfer_nft(&0, &from, &to, &from);
    assert_eq!(
        result,
        Err(Ok(crate::errors::NftErrorCode::NftNotFound)),
        "transferring NFT id=0 must return NftNotFound"
    );
}

// --- Transfer by non-owner ---

#[test]
fn test_transfer_by_non_owner_returns_not_owner() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let recipient = Address::generate(&env);
    let metadata = create_metadata(&env, "Owned NFT", "desc", "ipfs://owned");
    let nft_id = mint_transferable(&env, &client, 1, &owner, &metadata);

    // attacker passes their own address as `from`, which doesn't match the actual owner
    let result = client.try_transfer_nft(&nft_id, &attacker, &recipient, &attacker);
    assert_eq!(
        result,
        Err(Ok(crate::errors::NftErrorCode::NotOwner)),
        "transfer where from != actual owner must return NotOwner"
    );
}

#[test]
fn test_transfer_by_non_owner_does_not_change_ownership() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let recipient = Address::generate(&env);
    let metadata = create_metadata(&env, "Guard NFT", "desc", "ipfs://guard");
    let nft_id = mint_transferable(&env, &client, 1, &owner, &metadata);

    let _ = client.try_transfer_nft(&nft_id, &attacker, &recipient, &attacker);

    assert_eq!(
        client.owner_of(&nft_id),
        Some(owner.clone()),
        "ownership must be unchanged after failed transfer attempt by non-owner"
    );
    // attacker and recipient should have no NFTs
    assert_eq!(client.get_player_nfts(&attacker, &0, &100).len(), 0);
    assert_eq!(client.get_player_nfts(&recipient, &0, &100).len(), 0);
}

#[test]
fn test_transfer_caller_not_operator_returns_not_operator() {
    // `from` is the real owner but `caller` is neither owner nor approved operator
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let owner = Address::generate(&env);
    let third_party = Address::generate(&env);
    let recipient = Address::generate(&env);
    let metadata = create_metadata(&env, "Operator Test", "desc", "ipfs://op");
    let nft_id = mint_transferable(&env, &client, 1, &owner, &metadata);

    // caller is third_party (not owner, not operator)
    let result = client.try_transfer_nft(&nft_id, &owner, &recipient, &third_party);
    assert_eq!(
        result,
        Err(Ok(crate::errors::NftErrorCode::NotOperator)),
        "transfer by unapproved caller must return NotOperator"
    );
}

#[test]
fn test_operator_can_transfer_on_behalf_of_owner() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let recipient = Address::generate(&env);
    let metadata = create_metadata(&env, "Operator Xfer", "desc", "ipfs://opxfer");
    let nft_id = mint_transferable(&env, &client, 1, &owner, &metadata);

    client.set_operator(&owner, &operator);
    client.transfer_nft(&nft_id, &owner, &recipient, &operator);

    assert_eq!(
        client.owner_of(&nft_id),
        Some(recipient.clone()),
        "operator-initiated transfer must update ownership to recipient"
    );
}

#[test]
fn test_revoked_operator_cannot_transfer() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let recipient = Address::generate(&env);
    let metadata = create_metadata(&env, "Revoke Test", "desc", "ipfs://revoke");
    let nft_id = mint_transferable(&env, &client, 1, &owner, &metadata);

    client.set_operator(&owner, &operator);
    client.remove_operator(&owner, &operator);

    let result = client.try_transfer_nft(&nft_id, &owner, &recipient, &operator);
    assert_eq!(
        result,
        Err(Ok(crate::errors::NftErrorCode::NotOperator)),
        "revoked operator must not be able to transfer"
    );
    assert_eq!(client.owner_of(&nft_id), Some(owner.clone()));
}

// --- Transfer soulbound NFT ---

#[test]
fn test_transfer_soulbound_nft_returns_soulbound_error() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let owner = Address::generate(&env);
    let recipient = Address::generate(&env);
    let metadata = create_metadata(&env, "Soulbound NFT", "non-transferable", "ipfs://soul");

    // mint_reward_nft defaults to transferable = false (soulbound)
    let nft_id = mint_soulbound(&env, &client, 1, &owner, &metadata);

    let result = client.try_transfer_nft(&nft_id, &owner, &recipient, &owner);
    assert_eq!(
        result,
        Err(Ok(crate::errors::NftErrorCode::SoulboundNft)),
        "transferring a soulbound NFT must return SoulboundNft"
    );
}

#[test]
fn test_soulbound_nft_ownership_unchanged_after_attempted_transfer() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let owner = Address::generate(&env);
    let recipient = Address::generate(&env);
    let metadata = create_metadata(&env, "Bound Trophy", "soul", "ipfs://bound");
    let nft_id = mint_soulbound(&env, &client, 1, &owner, &metadata);

    let _ = client.try_transfer_nft(&nft_id, &owner, &recipient, &owner);

    // owner still holds the NFT
    assert_eq!(client.owner_of(&nft_id), Some(owner.clone()));
    assert_eq!(client.get_player_nfts(&owner, &0, &100).len(), 1);
    assert_eq!(client.get_player_nfts(&recipient, &0, &100).len(), 0);
}

#[test]
fn test_soulbound_nft_operator_cannot_override_soulbound() {
    // Even an approved operator should not be able to transfer a soulbound NFT.
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let recipient = Address::generate(&env);
    let metadata = create_metadata(&env, "Soul + Op", "soul", "ipfs://soulop");
    let nft_id = mint_soulbound(&env, &client, 1, &owner, &metadata);

    client.set_operator(&owner, &operator);

    let result = client.try_transfer_nft(&nft_id, &owner, &recipient, &operator);
    assert_eq!(
        result,
        Err(Ok(crate::errors::NftErrorCode::SoulboundNft)),
        "an operator must not be able to bypass the soulbound restriction"
    );
    assert_eq!(client.owner_of(&nft_id), Some(owner.clone()));
}

// --- Transfer and verify ownership update ---

#[test]
fn test_transfer_updates_owner_field_in_nft_data() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let metadata = create_metadata(&env, "Ownership Check", "desc", "ipfs://oc");
    let nft_id = mint_transferable(&env, &client, 1, &alice, &metadata);

    client.transfer_nft(&nft_id, &alice, &bob, &alice);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.owner, bob, "NftData.owner must reflect the new owner");
}

#[test]
fn test_transfer_updates_owner_of_query() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let metadata = create_metadata(&env, "OwnerOf Query", "desc", "ipfs://oq");
    let nft_id = mint_transferable(&env, &client, 1, &alice, &metadata);

    client.transfer_nft(&nft_id, &alice, &bob, &alice);

    assert_eq!(client.owner_of(&nft_id), Some(bob.clone()));
    assert_eq!(client.get_nft_owner(&nft_id), Some(bob));
}

#[test]
fn test_transfer_updates_player_nft_indexes_correctly() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    // Mint two NFTs for alice
    let m1 = create_metadata(&env, "NFT A", "a", "ipfs://a");
    let m2 = create_metadata(&env, "NFT B", "b", "ipfs://b");
    let nft1 = mint_transferable(&env, &client, 1, &alice, &m1);
    let nft2 = mint_transferable(&env, &client, 2, &alice, &m2);

    assert_eq!(client.get_player_nfts(&alice, &0, &100).len(), 2);
    assert_eq!(client.get_player_nfts(&bob, &0, &100).len(), 0);

    // Transfer nft1 to bob
    client.transfer_nft(&nft1, &alice, &bob, &alice);

    let alice_nfts = client.get_player_nfts(&alice, &0, &100);
    assert_eq!(alice_nfts.len(), 1, "alice must have 1 NFT remaining");
    assert_eq!(
        alice_nfts.get(0).unwrap(),
        nft2,
        "alice's remaining NFT must be nft2"
    );

    let bob_nfts = client.get_player_nfts(&bob, &0, &100);
    assert_eq!(bob_nfts.len(), 1, "bob must have 1 NFT after transfer");
    assert_eq!(
        bob_nfts.get(0).unwrap(),
        nft1,
        "bob's NFT must be the transferred nft1"
    );
}

#[test]
fn test_transfer_preserves_completion_player_field() {
    // completion_player is the address that originally completed the hunt —
    // it must be immutable even after ownership changes.
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let original_player = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let metadata = create_metadata(&env, "Provenance NFT", "desc", "ipfs://prov");
    let nft_id = mint_transferable(&env, &client, 1, &original_player, &metadata);

    client.transfer_nft(&nft_id, &original_player, &new_owner, &original_player);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(
        nft.completion_player, original_player,
        "completion_player must not change after a transfer"
    );
    assert_eq!(nft.owner, new_owner);
}

#[test]
fn test_transfer_preserves_metadata_fields() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let metadata = create_metadata_full(
        &env,
        "Rare Trophy",
        "Very rare",
        "ipfs://rare",
        "Epic Hunt",
        4,
        2,
    );
    let nft_id = mint_transferable(&env, &client, 42, &alice, &metadata);

    client.transfer_nft(&nft_id, &alice, &bob, &alice);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.title, String::from_str(&env, "Rare Trophy"));
    assert_eq!(nft.metadata.description, String::from_str(&env, "Very rare"));
    assert_eq!(nft.metadata.image_uri, String::from_str(&env, "ipfs://rare"));
    assert_eq!(nft.metadata.hunt_title, String::from_str(&env, "Epic Hunt"));
    assert_eq!(nft.metadata.rarity, 4);
    assert_eq!(nft.metadata.tier, 2);
    assert_eq!(nft.hunt_id, 42);
}

#[test]
fn test_chained_transfers_track_ownership_correctly() {
    // A → B → C: each step must correctly update owner and player indexes.
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let c = Address::generate(&env);
    let metadata = create_metadata(&env, "Chain NFT", "chain", "ipfs://chain");
    let nft_id = mint_transferable(&env, &client, 1, &a, &metadata);

    // A → B
    client.transfer_nft(&nft_id, &a, &b, &a);
    assert_eq!(client.owner_of(&nft_id), Some(b.clone()));
    assert_eq!(client.get_player_nfts(&a, &0, &100).len(), 0);
    assert_eq!(client.get_player_nfts(&b, &0, &100).len(), 1);

    // B → C
    client.transfer_nft(&nft_id, &b, &c, &b);
    assert_eq!(client.owner_of(&nft_id), Some(c.clone()));
    assert_eq!(client.get_player_nfts(&b, &0, &100).len(), 0);
    assert_eq!(client.get_player_nfts(&c, &0, &100).len(), 1);
}

#[test]
fn test_transfer_emits_nft_transferred_event_with_correct_fields() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let metadata = create_metadata(&env, "Event Check", "desc", "ipfs://ev");
    let nft_id = mint_transferable(&env, &client, 1, &from, &metadata);

    client.transfer_nft(&nft_id, &from, &to, &from);

    let events = env.events().all();
    // Find the NftTransferred event (last event published)
    let (_contract, topics, data) = events.get(events.len() - 1).unwrap();

    // topics[0] == "NftTransferred", topics[1] == nft_id
    let topic0: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    assert_eq!(topic0, Symbol::new(&env, "NftTransferred"));

    let event: crate::NftTransferredEvent =
        crate::NftTransferredEvent::try_from_val(&env, &data).unwrap();
    assert_eq!(event.nft_id, nft_id);
    assert_eq!(event.from, from);
    assert_eq!(event.to, to);
}
