#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, Vec, Address, symbol_short};

// Content structure to store details about social media content
#[contracttype]
#[derive(Clone)]
pub struct Content {
    pub content_id: u64,
    pub creator: Address,
    pub title: String,
    pub content_type: String, // e.g., "post", "video", "article"
    pub timestamp: u64,
    pub likes: u64,
    pub shares: u64,
    pub comments: u64,
    pub is_active: bool,
}

// User profile structure to store user reputation and rewards
#[contracttype]
#[derive(Clone)]
pub struct UserProfile {
    pub user_addr: Address,
    pub reputation_score: u64,
    pub total_rewards: u64,
    pub content_count: u64,
    pub joined_timestamp: u64,
}

// RewardMetrics structure to track platform-wide statistics
#[contracttype]
#[derive(Clone)]
pub struct RewardMetrics {
    pub total_content: u64,
    pub total_users: u64,
    pub total_rewards_distributed: u64,
    pub active_content: u64,
}

// Mapping for user's content list
#[contracttype]
pub enum UserContentList {
    ForUser(Address)
}

// Mapping for individual content
#[contracttype]
pub enum ContentMap {
    Content(u64)
}

// Mapping for user profiles
#[contracttype]
pub enum UserProfileMap {
    Profile(Address)
}

// Constants for storage keys
const REWARD_METRICS: Symbol = symbol_short!("R_METRICS");
const CONTENT_COUNT: Symbol = symbol_short!("C_COUNT");

#[contract]
pub struct SocialMediaRewardsContract;

#[contractimpl]
impl SocialMediaRewardsContract {
    // Get content by ID
    pub fn get_content(env: Env, content_id: u64) -> Content {
        env.storage().instance().get(&ContentMap::Content(content_id)).unwrap_or(Content {
            content_id: 0,
            creator: Address::from_string(&String::from_str(&env, "")),
            title: String::from_str(&env, ""),
            content_type: String::from_str(&env, ""),
            timestamp: 0,
            likes: 0,
            shares: 0,
            comments: 0,
            is_active: false,
        })
    }
    
    // Get user profile 
    pub fn get_user_profile(env: Env, user: Address) -> UserProfile {
        env.storage().instance().get(&UserProfileMap::Profile(user.clone())).unwrap_or(UserProfile {
            user_addr: user,
            reputation_score: 0,
            total_rewards: 0,
            content_count: 0,
            joined_timestamp: 0,
        })
    }
    
    // Get list of content IDs created by a user
    pub fn get_user_content_list(env: Env, user: Address) -> Vec<u64> {
        env.storage().instance().get(&UserContentList::ForUser(user)).unwrap_or(Vec::new(&env))
    }
    
    // Get platform-wide reward metrics
    pub fn get_reward_metrics(env: Env) -> RewardMetrics {
        env.storage().instance().get(&REWARD_METRICS).unwrap_or(RewardMetrics {
            total_content: 0,
            total_users: 0,
            total_rewards_distributed: 0,
            active_content: 0,
        })
    }
    
    // Create new content and register it on the blockchain
    pub fn create_content(
        env: Env, 
        creator: Address, 
        title: String, 
        content_type: String
    ) -> u64 {
        // Authenticate the creator
        creator.require_auth();
        
        // Get current content count
        let mut content_count: u64 = env.storage().instance().get(&CONTENT_COUNT).unwrap_or(0);
        content_count += 1;
        
        // Get current timestamp
        let timestamp = env.ledger().timestamp();
        
        // Create new content
        let content = Content {
            content_id: content_count,
            creator: creator.clone(),
            title,
            content_type,
            timestamp,
            likes: 0,
            shares: 0,
            comments: 0,
            is_active: true,
        };
        
        // Store content in blockchain
        env.storage().instance().set(&ContentMap::Content(content_count), &content);
        
        // Update user's content list
        let mut user_content_list: Vec<u64> = SocialMediaRewardsContract::get_user_content_list(env.clone(), creator.clone());
        user_content_list.push_back(content_count);
        env.storage().instance().set(&UserContentList::ForUser(creator.clone()), &user_content_list);
        
        // Update user profile
        let mut user_profile = SocialMediaRewardsContract::get_user_profile(env.clone(), creator.clone());
        user_profile.content_count += 1;
        env.storage().instance().set(&UserProfileMap::Profile(creator.clone()), &user_profile);
        
        // Update platform metrics
        let mut metrics = SocialMediaRewardsContract::get_reward_metrics(env.clone());
        metrics.total_content += 1;
        metrics.active_content += 1;
        env.storage().instance().set(&REWARD_METRICS, &metrics);
        
        // Update content count
        env.storage().instance().set(&CONTENT_COUNT, &content_count);
        
        // Extend contract storage TTL
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Content created with ID: {}", content_count);
        content_count
    }
    
    // Record engagement (likes, shares, comments) and distribute rewards
    pub fn record_engagement(
        env: Env, 
        user: Address, 
        content_id: u64, 
        engagement_type: String
    ) -> u64 {
        // Authenticate the user
        user.require_auth();
        
        // Get content
        let mut content = SocialMediaRewardsContract::get_content(env.clone(), content_id);
        
        // Verify content exists and is active
        if content.content_id == 0 || !content.is_active {
            log!(&env, "Content not found or not active");
            panic!("Content not found or not active");
        }
        
        // Update engagement metrics based on type
        let reward_amount: u64 = if engagement_type == String::from_str(&env, "like") {
            content.likes += 1;
            1 // 1 token for like
        } else if engagement_type == String::from_str(&env, "share") {
            content.shares += 1;
            3 // 3 tokens for share
        } else if engagement_type == String::from_str(&env, "comment") {
            content.comments += 1;
            2 // 2 tokens for comment
        } else {
            log!(&env, "Invalid engagement type");
            panic!("Invalid engagement type");
        };
        
        // Update content
        env.storage().instance().set(&ContentMap::Content(content_id), &content);
        
        // Update creator's profile with rewards
        let content_creator = content.creator;
        let mut creator_profile = SocialMediaRewardsContract::get_user_profile(env.clone(), content_creator.clone());
        creator_profile.total_rewards += reward_amount;
        
        // Reputation boost based on engagement
        creator_profile.reputation_score += reward_amount;
        env.storage().instance().set(&UserProfileMap::Profile(content_creator), &creator_profile);
        
        // Update platform metrics
        let mut metrics = SocialMediaRewardsContract::get_reward_metrics(env.clone());
        metrics.total_rewards_distributed += reward_amount;
        env.storage().instance().set(&REWARD_METRICS, &metrics);
        
        // Extend contract storage TTL
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Engagement recorded: {} on content: {}", engagement_type, content_id);
        log!(&env, "Rewards distributed: {}", reward_amount);
        
        reward_amount
    }
    
    // Register a new user with the platform
    pub fn register_user(env: Env, user: Address) -> UserProfile {
        // Authenticate the user
        user.require_auth();
        
        // Check if user already exists
        let existing_profile = SocialMediaRewardsContract::get_user_profile(env.clone(), user.clone());
        if existing_profile.user_addr == user {
            log!(&env, "User already registered");
            panic!("User already registered");
        }
        
        // Get current timestamp
        let timestamp = env.ledger().timestamp();
        
        // Create new user profile
        let profile = UserProfile {
            user_addr: user.clone(),
            reputation_score: 10, // Starting reputation
            total_rewards: 0,
            content_count: 0,
            joined_timestamp: timestamp,
        };
        
        // Store user profile
        env.storage().instance().set(&UserProfileMap::Profile(user.clone()), &profile);
        
        // Initialize empty content list for user
        let empty_list: Vec<u64> = Vec::new(&env);
        env.storage().instance().set(&UserContentList::ForUser(user.clone()), &empty_list);
        
        // Update platform metrics
        let mut metrics = SocialMediaRewardsContract::get_reward_metrics(env.clone());
        metrics.total_users += 1;
        env.storage().instance().set(&REWARD_METRICS, &metrics);
        
        // Extend contract storage TTL
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "User registered: {}", user);
        profile
    }
    
    // Deactivate content (removing from platform but keeping record)
    pub fn deactivate_content(env: Env, creator: Address, content_id: u64) {
        // Authenticate the creator
        creator.require_auth();
        
        // Get content
        let mut content = SocialMediaRewardsContract::get_content(env.clone(), content_id);
        
        // Verify content exists and creator owns it
        if content.content_id == 0 {
            log!(&env, "Content not found");
            panic!("Content not found");
        }
        
        if content.creator != creator {
            log!(&env, "Only the creator can deactivate content");
            panic!("Only the creator can deactivate content");
        }
        
        // Deactivate content
        content.is_active = false;
        env.storage().instance().set(&ContentMap::Content(content_id), &content);
        
        // Update platform metrics
        let mut metrics = SocialMediaRewardsContract::get_reward_metrics(env.clone());
        metrics.active_content -= 1;
        env.storage().instance().set(&REWARD_METRICS, &metrics);
        
        // Extend contract storage TTL
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Content deactivated: {}", content_id);
    }
}