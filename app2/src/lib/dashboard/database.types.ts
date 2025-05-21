export type Json =
  | string
  | number
  | boolean
  | null
  | { [key: string]: Json | undefined }
  | Json[]

export type Database = {
  public: {
    Tables: {
      achievement_types: {
        Row: {
          created_at: string
          description: string
          id: number
          schema: Json | null
        }
        Insert: {
          created_at?: string
          description: string
          id: number
          schema?: Json | null
        }
        Update: {
          created_at?: string
          description?: string
          id?: number
          schema?: Json | null
        }
        Relationships: []
      }
      achievements: {
        Row: {
          category: number | null
          created_at: string
          description: string | null
          id: number
          image: string | null
          meta: Json | null
          next: number | null
          priority: number
          public: boolean
          subcategory: number | null
          tags: string[] | null
          tenant_id: number | null
          title: string | null
          type: number | null
          xp: number | null
        }
        Insert: {
          category?: number | null
          created_at?: string
          description?: string | null
          id?: number
          image?: string | null
          meta?: Json | null
          next?: number | null
          priority?: number
          public?: boolean
          subcategory?: number | null
          tags?: string[] | null
          tenant_id?: number | null
          title?: string | null
          type?: number | null
          xp?: number | null
        }
        Update: {
          category?: number | null
          created_at?: string
          description?: string | null
          id?: number
          image?: string | null
          meta?: Json | null
          next?: number | null
          priority?: number
          public?: boolean
          subcategory?: number | null
          tags?: string[] | null
          tenant_id?: number | null
          title?: string | null
          type?: number | null
          xp?: number | null
        }
        Relationships: [
          {
            foreignKeyName: "achievements_category_fkey"
            columns: ["category"]
            isOneToOne: false
            referencedRelation: "categories"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "achievements_next_fkey"
            columns: ["next"]
            isOneToOne: false
            referencedRelation: "achievements"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "achievements_subcategory_fkey"
            columns: ["subcategory"]
            isOneToOne: false
            referencedRelation: "categories"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "achievements_tenant_id_fkey"
            columns: ["tenant_id"]
            isOneToOne: false
            referencedRelation: "tentants"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "achievements_type_fkey"
            columns: ["type"]
            isOneToOne: false
            referencedRelation: "achievement_types"
            referencedColumns: ["id"]
          },
        ]
      }
      aggregate_transfer_counts: {
        Row: {
          count: number
          destination_chain: string
          phase: string | null
          universal_chain_id: string | null
          user_id: string
        }
        Insert: {
          count: number
          destination_chain: string
          phase?: string | null
          universal_chain_id?: string | null
          user_id: string
        }
        Update: {
          count?: number
          destination_chain?: string
          phase?: string | null
          universal_chain_id?: string | null
          user_id?: string
        }
        Relationships: [
          {
            foreignKeyName: "aggregate_transfer_counts_user_id_fkey"
            columns: ["user_id"]
            isOneToOne: false
            referencedRelation: "user_discord_invites"
            referencedColumns: ["user_id"]
          },
          {
            foreignKeyName: "aggregate_transfer_counts_user_id_fkey"
            columns: ["user_id"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
        ]
      }
      alerts: {
        Row: {
          created_at: string
          enable: boolean | null
          id: number
          message: string | null
        }
        Insert: {
          created_at?: string
          enable?: boolean | null
          id?: number
          message?: string | null
        }
        Update: {
          created_at?: string
          enable?: boolean | null
          id?: number
          message?: string | null
        }
        Relationships: []
      }
      call_attendees: {
        Row: {
          call: string
          created_at: string
          discord_id: number
        }
        Insert: {
          call?: string
          created_at?: string
          discord_id?: number
        }
        Update: {
          call?: string
          created_at?: string
          discord_id?: number
        }
        Relationships: []
      }
      categories: {
        Row: {
          created_at: string
          description: string | null
          id: number
          title: string
        }
        Insert: {
          created_at?: string
          description?: string | null
          id?: number
          title: string
        }
        Update: {
          created_at?: string
          description?: string | null
          id?: number
          title?: string
        }
        Relationships: []
      }
      chains: {
        Row: {
          chain_id: string
          created_at: string
          enabled: boolean
          logo_uri: string | null
          name: string
          prefix: string | null
          public: boolean
          testnet: boolean
          type: string | null
        }
        Insert: {
          chain_id: string
          created_at?: string
          enabled?: boolean
          logo_uri?: string | null
          name: string
          prefix?: string | null
          public?: boolean
          testnet?: boolean
          type?: string | null
        }
        Update: {
          chain_id?: string
          created_at?: string
          enabled?: boolean
          logo_uri?: string | null
          name?: string
          prefix?: string | null
          public?: boolean
          testnet?: boolean
          type?: string | null
        }
        Relationships: []
      }
      days_interacted: {
        Row: {
          days: number
          destination_universal_chain_id: string
          last_updated_at: string
          new_days: number | null
          user_id: string
        }
        Insert: {
          days: number
          destination_universal_chain_id: string
          last_updated_at?: string
          new_days?: number | null
          user_id: string
        }
        Update: {
          days?: number
          destination_universal_chain_id?: string
          last_updated_at?: string
          new_days?: number | null
          user_id?: string
        }
        Relationships: []
      }
      discord_guilds: {
        Row: {
          created_at: string
          description: string
          id: number
          name: string
        }
        Insert: {
          created_at?: string
          description: string
          id: number
          name: string
        }
        Update: {
          created_at?: string
          description?: string
          id?: number
          name?: string
        }
        Relationships: []
      }
      discord_invites: {
        Row: {
          code: string
          created_at: string
          guild_id: number
          inviter: number
          meta: Json
          type: number
        }
        Insert: {
          code: string
          created_at?: string
          guild_id: number
          inviter: number
          meta: Json
          type: number
        }
        Update: {
          code?: string
          created_at?: string
          guild_id?: number
          inviter?: number
          meta?: Json
          type?: number
        }
        Relationships: []
      }
      discord_roles: {
        Row: {
          guild_id: number
          id: number
          meta: Json
          name: string
        }
        Insert: {
          guild_id: number
          id: number
          meta: Json
          name: string
        }
        Update: {
          guild_id?: number
          id?: number
          meta?: Json
          name?: string
        }
        Relationships: [
          {
            foreignKeyName: "discord_roles_guild_id_fkey"
            columns: ["guild_id"]
            isOneToOne: false
            referencedRelation: "discord_guilds"
            referencedColumns: ["id"]
          },
        ]
      }
      discord_server_members: {
        Row: {
          guild_id: number
          meta: Json | null
          updated_at: string
          user_id: number
          username: string | null
        }
        Insert: {
          guild_id: number
          meta?: Json | null
          updated_at?: string
          user_id: number
          username?: string | null
        }
        Update: {
          guild_id?: number
          meta?: Json | null
          updated_at?: string
          user_id?: number
          username?: string | null
        }
        Relationships: [
          {
            foreignKeyName: "discord_server_members_guild_id_fkey"
            columns: ["guild_id"]
            isOneToOne: false
            referencedRelation: "discord_guilds"
            referencedColumns: ["id"]
          },
        ]
      }
      fudders: {
        Row: {
          reason: string
          twitter_id: number
        }
        Insert: {
          reason: string
          twitter_id: number
        }
        Update: {
          reason?: string
          twitter_id?: number
        }
        Relationships: []
      }
      functions: {
        Row: {
          created_at: string
          name: string
        }
        Insert: {
          created_at?: string
          name: string
        }
        Update: {
          created_at?: string
          name?: string
        }
        Relationships: []
      }
      github_contributors: {
        Row: {
          id: number
          meta: Json
          owner: string
          repo: string
          username: string
        }
        Insert: {
          id: number
          meta: Json
          owner: string
          repo: string
          username: string
        }
        Update: {
          id?: number
          meta?: Json
          owner?: string
          repo?: string
          username?: string
        }
        Relationships: []
      }
      github_stargazers: {
        Row: {
          created_at: string
          id: number
          meta: Json
          owner: string
          repo: string
          username: string
        }
        Insert: {
          created_at?: string
          id: number
          meta: Json
          owner: string
          repo: string
          username: string
        }
        Update: {
          created_at?: string
          id?: number
          meta?: Json
          owner?: string
          repo?: string
          username?: string
        }
        Relationships: []
      }
      identities_to_delete: {
        Row: {
          created_at: string
          deleted_at: string | null
          id: string
          user_id: string
        }
        Insert: {
          created_at?: string
          deleted_at?: string | null
          id: string
          user_id: string
        }
        Update: {
          created_at?: string
          deleted_at?: string | null
          id?: string
          user_id?: string
        }
        Relationships: []
      }
      json_schemas: {
        Row: {
          created_at: string
          description: string
          id: number
          schema: Json
        }
        Insert: {
          created_at?: string
          description: string
          id?: number
          schema: Json
        }
        Update: {
          created_at?: string
          description?: string
          id?: number
          schema?: Json
        }
        Relationships: []
      }
      kaito_vote: {
        Row: {
          "% of TOTAL": string | null
          "100XP": string | null
          "earliest_smart_follower_votes_timestamp": string | null
          "earliest_yap_votes_timestamp": string | null
          "follower": number | null
          "lifetime_yaps": string | null
          "smart_follower": number | null
          "TOTAL AVAILABLE VOTES": number | null
          "TOTAL VOTES TO UNION": number | null
          "total_smart_follower_votes": number | null
          "total_votes": number | null
          "total_yap_votes": string | null
          "twitter_id": number
          "twitter_username": string | null
          "weighted_smart_follower_votes_timestamp": string | null
          "weighted_yap_votes_timestamp": string | null
          "WWS NFT": string | null
        }
        Insert: {
          "% of TOTAL"?: string | null
          "100XP"?: string | null
          "earliest_smart_follower_votes_timestamp"?: string | null
          "earliest_yap_votes_timestamp"?: string | null
          "follower"?: number | null
          "lifetime_yaps"?: string | null
          "smart_follower"?: number | null
          "TOTAL AVAILABLE VOTES"?: number | null
          "TOTAL VOTES TO UNION"?: number | null
          "total_smart_follower_votes"?: number | null
          "total_votes"?: number | null
          "total_yap_votes"?: string | null
          "twitter_id": number
          "twitter_username"?: string | null
          "weighted_smart_follower_votes_timestamp"?: string | null
          "weighted_yap_votes_timestamp"?: string | null
          "WWS NFT"?: string | null
        }
        Update: {
          "% of TOTAL"?: string | null
          "100XP"?: string | null
          "earliest_smart_follower_votes_timestamp"?: string | null
          "earliest_yap_votes_timestamp"?: string | null
          "follower"?: number | null
          "lifetime_yaps"?: string | null
          "smart_follower"?: number | null
          "TOTAL AVAILABLE VOTES"?: number | null
          "TOTAL VOTES TO UNION"?: number | null
          "total_smart_follower_votes"?: number | null
          "total_votes"?: number | null
          "total_yap_votes"?: string | null
          "twitter_id"?: number
          "twitter_username"?: string | null
          "weighted_smart_follower_votes_timestamp"?: string | null
          "weighted_yap_votes_timestamp"?: string | null
          "WWS NFT"?: string | null
        }
        Relationships: []
      }
      levels: {
        Row: {
          created_at: string
          experience_required: number | null
          id: number
          title: string | null
        }
        Insert: {
          created_at?: string
          experience_required?: number | null
          id?: number
          title?: string | null
        }
        Update: {
          created_at?: string
          experience_required?: number | null
          id?: number
          title?: string | null
        }
        Relationships: []
      }
      logs: {
        Row: {
          by: string
          created_at: string
          data: string
          id: string
        }
        Insert: {
          by: string
          created_at?: string
          data: string
          id: string
        }
        Update: {
          by?: string
          created_at?: string
          data?: string
          id?: string
        }
        Relationships: []
      }
      mission_rewards: {
        Row: {
          created_at: string
          mission_id: number
          reward_id: number
        }
        Insert: {
          created_at?: string
          mission_id: number
          reward_id: number
        }
        Update: {
          created_at?: string
          mission_id?: number
          reward_id?: number
        }
        Relationships: [
          {
            foreignKeyName: "mission_rewards_mission_id_fkey"
            columns: ["mission_id"]
            isOneToOne: false
            referencedRelation: "missions"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "mission_rewards_reward_id_fkey"
            columns: ["reward_id"]
            isOneToOne: false
            referencedRelation: "rewards"
            referencedColumns: ["id"]
          },
        ]
      }
      mission_types: {
        Row: {
          created_at: string
          description: string
          id: number
          schema: Json | null
        }
        Insert: {
          created_at?: string
          description: string
          id: number
          schema?: Json | null
        }
        Update: {
          created_at?: string
          description?: string
          id?: number
          schema?: Json | null
        }
        Relationships: []
      }
      missions: {
        Row: {
          category: number | null
          created_at: string
          description: string
          end: string
          id: number
          meta: Json
          priority: number
          public: boolean
          start: string
          subcategory: number | null
          title: string
          type: number
          xp: number
        }
        Insert: {
          category?: number | null
          created_at?: string
          description: string
          end: string
          id?: number
          meta: Json
          priority?: number
          public?: boolean
          start: string
          subcategory?: number | null
          title: string
          type: number
          xp?: number
        }
        Update: {
          category?: number | null
          created_at?: string
          description?: string
          end?: string
          id?: number
          meta?: Json
          priority?: number
          public?: boolean
          start?: string
          subcategory?: number | null
          title?: string
          type?: number
          xp?: number
        }
        Relationships: [
          {
            foreignKeyName: "missions_category_fkey"
            columns: ["category"]
            isOneToOne: false
            referencedRelation: "categories"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "missions_subcategory_fkey"
            columns: ["subcategory"]
            isOneToOne: false
            referencedRelation: "categories"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "missions_type_fkey"
            columns: ["type"]
            isOneToOne: false
            referencedRelation: "mission_types"
            referencedColumns: ["id"]
          },
        ]
      }
      nfts: {
        Row: {
          collection_id: string
          created_at: string
          provider: string
          token_id: string | null
          updated_at: string
          user_id: string
        }
        Insert: {
          collection_id: string
          created_at?: string
          provider: string
          token_id?: string | null
          updated_at?: string
          user_id?: string
        }
        Update: {
          collection_id?: string
          created_at?: string
          provider?: string
          token_id?: string | null
          updated_at?: string
          user_id?: string
        }
        Relationships: []
      }
      partners: {
        Row: {
          created_at: string
          id: number
          name: string
          points: number
          points_used: number
          webhook: string
        }
        Insert: {
          created_at?: string
          id?: number
          name: string
          points: number
          points_used?: number
          webhook: string
        }
        Update: {
          created_at?: string
          id?: number
          name?: string
          points?: number
          points_used?: number
          webhook?: string
        }
        Relationships: []
      }
      phased_transfer_counts: {
        Row: {
          count: number
          new_count: number | null
          phase: string
          universal_chain_id: string
          user_id: string
        }
        Insert: {
          count: number
          new_count?: number | null
          phase: string
          universal_chain_id: string
          user_id?: string
        }
        Update: {
          count?: number
          new_count?: number | null
          phase?: string
          universal_chain_id?: string
          user_id?: string
        }
        Relationships: []
      }
      resend_audiences: {
        Row: {
          created_at: string
          id: string
          name: string
        }
        Insert: {
          created_at?: string
          id?: string
          name: string
        }
        Update: {
          created_at?: string
          id?: string
          name?: string
        }
        Relationships: []
      }
      resend_email_contacts: {
        Row: {
          audience_id: string
          contact_id: string
          created_at: string
          email: string
        }
        Insert: {
          audience_id: string
          contact_id: string
          created_at?: string
          email: string
        }
        Update: {
          audience_id?: string
          contact_id?: string
          created_at?: string
          email?: string
        }
        Relationships: []
      }
      reward_achievements: {
        Row: {
          achievement_id: number
          created_at: string
          reward_id: number
        }
        Insert: {
          achievement_id: number
          created_at?: string
          reward_id?: number
        }
        Update: {
          achievement_id?: number
          created_at?: string
          reward_id?: number
        }
        Relationships: [
          {
            foreignKeyName: "reward_achievements_achievement_id_fkey"
            columns: ["achievement_id"]
            isOneToOne: false
            referencedRelation: "achievements"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "reward_achievements_reward_id_fkey"
            columns: ["reward_id"]
            isOneToOne: false
            referencedRelation: "rewards"
            referencedColumns: ["id"]
          },
        ]
      }
      rewards: {
        Row: {
          created_at: string
          cutoff: string | null
          default_handled: boolean
          default_requires_handling: boolean
          description: string | null
          id: number
          meta: Json | null
          title: string | null
          type: number | null
        }
        Insert: {
          created_at?: string
          cutoff?: string | null
          default_handled?: boolean
          default_requires_handling?: boolean
          description?: string | null
          id?: number
          meta?: Json | null
          title?: string | null
          type?: number | null
        }
        Update: {
          created_at?: string
          cutoff?: string | null
          default_handled?: boolean
          default_requires_handling?: boolean
          description?: string | null
          id?: number
          meta?: Json | null
          title?: string | null
          type?: number | null
        }
        Relationships: []
      }
      roles: {
        Row: {
          created_at: string
          id: number
          title: string
        }
        Insert: {
          created_at?: string
          id?: number
          title: string
        }
        Update: {
          created_at?: string
          id?: number
          title?: string
        }
        Relationships: []
      }
      streaks_by_chain: {
        Row: {
          destination_universal_chain_id: string
          end_timestamp: string | null
          phase: string
          seconds: number
          start_timestamp: string
          user_id: string
        }
        Insert: {
          destination_universal_chain_id: string
          end_timestamp?: string | null
          phase?: string
          seconds: number
          start_timestamp: string
          user_id?: string
        }
        Update: {
          destination_universal_chain_id?: string
          end_timestamp?: string | null
          phase?: string
          seconds?: number
          start_timestamp?: string
          user_id?: string
        }
        Relationships: [
          {
            foreignKeyName: "streaks_by_chain_user_id_fkey"
            columns: ["user_id"]
            isOneToOne: false
            referencedRelation: "user_discord_invites"
            referencedColumns: ["user_id"]
          },
          {
            foreignKeyName: "streaks_by_chain_user_id_fkey"
            columns: ["user_id"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
        ]
      }
      tentants: {
        Row: {
          created_at: string
          description: string
          id: number
          name: string
        }
        Insert: {
          created_at?: string
          description: string
          id?: number
          name: string
        }
        Update: {
          created_at?: string
          description?: string
          id?: number
          name?: string
        }
        Relationships: []
      }
      tracked_nfts: {
        Row: {
          collection_id: string
          name: string | null
          provider: string | null
        }
        Insert: {
          collection_id: string
          name?: string | null
          provider?: string | null
        }
        Update: {
          collection_id?: string
          name?: string | null
          provider?: string | null
        }
        Relationships: []
      }
      transfer_counts: {
        Row: {
          address: string
          count: number | null
          destination_chain: string
          source_chain: string
          user_id: string | null
        }
        Insert: {
          address: string
          count?: number | null
          destination_chain: string
          source_chain: string
          user_id?: string | null
        }
        Update: {
          address?: string
          count?: number | null
          destination_chain?: string
          source_chain?: string
          user_id?: string | null
        }
        Relationships: [
          {
            foreignKeyName: "transfer_counts_user_id_fkey"
            columns: ["user_id"]
            isOneToOne: false
            referencedRelation: "user_discord_invites"
            referencedColumns: ["user_id"]
          },
          {
            foreignKeyName: "transfer_counts_user_id_fkey"
            columns: ["user_id"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
        ]
      }
      tsc_contributors: {
        Row: {
          created_at: string
          email_used: string
          success: boolean
          user_id: string | null
          wallet: string | null
        }
        Insert: {
          created_at?: string
          email_used: string
          success: boolean
          user_id?: string | null
          wallet?: string | null
        }
        Update: {
          created_at?: string
          email_used?: string
          success?: boolean
          user_id?: string | null
          wallet?: string | null
        }
        Relationships: [
          {
            foreignKeyName: "tsc_contributors_user_id_fkey"
            columns: ["user_id"]
            isOneToOne: false
            referencedRelation: "user_discord_invites"
            referencedColumns: ["user_id"]
          },
          {
            foreignKeyName: "tsc_contributors_user_id_fkey"
            columns: ["user_id"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
        ]
      }
      twitter_followers: {
        Row: {
          follower_id: number
          leader_id: number
        }
        Insert: {
          follower_id: number
          leader_id: number
        }
        Update: {
          follower_id?: number
          leader_id?: number
        }
        Relationships: [
          {
            foreignKeyName: "twitter_followers_leader_id_fkey"
            columns: ["leader_id"]
            isOneToOne: false
            referencedRelation: "twitter_leaders"
            referencedColumns: ["twitter_id"]
          },
        ]
      }
      twitter_leaders: {
        Row: {
          created_at: string
          description: string
          exit_on_duplicate: boolean
          last_scraped: string
          screen_name: string
          twitter_id: number
        }
        Insert: {
          created_at?: string
          description: string
          exit_on_duplicate?: boolean
          last_scraped?: string
          screen_name: string
          twitter_id: number
        }
        Update: {
          created_at?: string
          description?: string
          exit_on_duplicate?: boolean
          last_scraped?: string
          screen_name?: string
          twitter_id?: number
        }
        Relationships: []
      }
      user_achievements: {
        Row: {
          achieved_at: string | null
          achievement_id: number
          created_at: string | null
          progression: number
          threshold: number
          user_id: string
        }
        Insert: {
          achieved_at?: string | null
          achievement_id: number
          created_at?: string | null
          progression?: number
          threshold?: number
          user_id?: string
        }
        Update: {
          achieved_at?: string | null
          achievement_id?: number
          created_at?: string | null
          progression?: number
          threshold?: number
          user_id?: string
        }
        Relationships: [
          {
            foreignKeyName: "user_achievements_achievement_id_fkey"
            columns: ["achievement_id"]
            isOneToOne: false
            referencedRelation: "achievements"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "user_achievements_user_id_fkey"
            columns: ["user_id"]
            isOneToOne: false
            referencedRelation: "user_discord_invites"
            referencedColumns: ["user_id"]
          },
          {
            foreignKeyName: "user_achievements_user_id_fkey"
            columns: ["user_id"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
        ]
      }
      user_discord_roles: {
        Row: {
          assign: boolean
          assign_response: Json | null
          created_at: string
          guild_id: number
          handled_at: string | null
          notify: boolean
          role_id: number
          template_message: string | null
          user_id: string
        }
        Insert: {
          assign?: boolean
          assign_response?: Json | null
          created_at?: string
          guild_id: number
          handled_at?: string | null
          notify?: boolean
          role_id: number
          template_message?: string | null
          user_id?: string
        }
        Update: {
          assign?: boolean
          assign_response?: Json | null
          created_at?: string
          guild_id?: number
          handled_at?: string | null
          notify?: boolean
          role_id?: number
          template_message?: string | null
          user_id?: string
        }
        Relationships: [
          {
            foreignKeyName: "user_discord_roles_guild_id_fkey"
            columns: ["guild_id"]
            isOneToOne: false
            referencedRelation: "discord_guilds"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "user_discord_roles_guild_id_role_id_fkey"
            columns: ["guild_id", "role_id"]
            isOneToOne: false
            referencedRelation: "discord_roles"
            referencedColumns: ["guild_id", "id"]
          },
          {
            foreignKeyName: "user_discord_roles_user_id_fkey"
            columns: ["user_id"]
            isOneToOne: false
            referencedRelation: "user_discord_invites"
            referencedColumns: ["user_id"]
          },
          {
            foreignKeyName: "user_discord_roles_user_id_fkey"
            columns: ["user_id"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
        ]
      }
      user_missions: {
        Row: {
          completed_at: string
          created_at: string
          mission_id: number
          progression: number
          threshold: number
          user_id: string
        }
        Insert: {
          completed_at?: string
          created_at?: string
          mission_id: number
          progression?: number
          threshold: number
          user_id?: string
        }
        Update: {
          completed_at?: string
          created_at?: string
          mission_id?: number
          progression?: number
          threshold?: number
          user_id?: string
        }
        Relationships: [
          {
            foreignKeyName: "user_missions_mission_id_fkey"
            columns: ["mission_id"]
            isOneToOne: false
            referencedRelation: "missions"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "user_missions_user_id_fkey"
            columns: ["user_id"]
            isOneToOne: false
            referencedRelation: "user_discord_invites"
            referencedColumns: ["user_id"]
          },
          {
            foreignKeyName: "user_missions_user_id_fkey"
            columns: ["user_id"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
        ]
      }
      user_points: {
        Row: {
          created_at: string
          end: string
          partner_id: number
          points: number
          reason: string
          start: string
          user_id: string
        }
        Insert: {
          created_at?: string
          end: string
          partner_id: number
          points: number
          reason: string
          start: string
          user_id?: string
        }
        Update: {
          created_at?: string
          end?: string
          partner_id?: number
          points?: number
          reason?: string
          start?: string
          user_id?: string
        }
        Relationships: [
          {
            foreignKeyName: "user_points_partner_id_fkey"
            columns: ["partner_id"]
            isOneToOne: false
            referencedRelation: "partners"
            referencedColumns: ["id"]
          },
        ]
      }
      user_rewards: {
        Row: {
          achievement_id: number | null
          created_at: string
          handled: boolean
          last_retried_at: string
          manually_assigned: boolean | null
          msg_id: number | null
          next_request_at: string
          queue_name: string | null
          requires_handling: boolean
          reward_id: number
          user_id: string
        }
        Insert: {
          achievement_id?: number | null
          created_at?: string
          handled?: boolean
          last_retried_at?: string
          manually_assigned?: boolean | null
          msg_id?: number | null
          next_request_at?: string
          queue_name?: string | null
          requires_handling?: boolean
          reward_id?: number
          user_id?: string
        }
        Update: {
          achievement_id?: number | null
          created_at?: string
          handled?: boolean
          last_retried_at?: string
          manually_assigned?: boolean | null
          msg_id?: number | null
          next_request_at?: string
          queue_name?: string | null
          requires_handling?: boolean
          reward_id?: number
          user_id?: string
        }
        Relationships: [
          {
            foreignKeyName: "user_rewards_achievement_id_user_id_fkey"
            columns: ["achievement_id", "user_id"]
            isOneToOne: false
            referencedRelation: "user_achievements"
            referencedColumns: ["achievement_id", "user_id"]
          },
          {
            foreignKeyName: "user_rewards_reward_id_fkey"
            columns: ["reward_id"]
            isOneToOne: false
            referencedRelation: "rewards"
            referencedColumns: ["id"]
          },
        ]
      }
      users: {
        Row: {
          discord_id: number | null
          display_name: string | null
          github_id: number | null
          id: string
          last_discord_update: string
          last_points_update: string
          last_update_tick: string
          pfp: string | null
          role_id: number
          ticked: boolean
          twitter_id: number | null
        }
        Insert: {
          discord_id?: number | null
          display_name?: string | null
          github_id?: number | null
          id?: string
          last_discord_update?: string
          last_points_update?: string
          last_update_tick?: string
          pfp?: string | null
          role_id?: number
          ticked?: boolean
          twitter_id?: number | null
        }
        Update: {
          discord_id?: number | null
          display_name?: string | null
          github_id?: number | null
          id?: string
          last_discord_update?: string
          last_points_update?: string
          last_update_tick?: string
          pfp?: string | null
          role_id?: number
          ticked?: boolean
          twitter_id?: number | null
        }
        Relationships: [
          {
            foreignKeyName: "users_role_id_fkey"
            columns: ["role_id"]
            isOneToOne: false
            referencedRelation: "roles"
            referencedColumns: ["id"]
          },
        ]
      }
      wallets: {
        Row: {
          address: string
          chain_id: string
          created_at: string
          grouping: string | null
          user_id: string
        }
        Insert: {
          address: string
          chain_id: string
          created_at?: string
          grouping?: string | null
          user_id?: string
        }
        Update: {
          address?: string
          chain_id?: string
          created_at?: string
          grouping?: string | null
          user_id?: string
        }
        Relationships: [
          {
            foreignKeyName: "wallets_user_id_fkey"
            columns: ["user_id"]
            isOneToOne: false
            referencedRelation: "user_discord_invites"
            referencedColumns: ["user_id"]
          },
          {
            foreignKeyName: "wallets_user_id_fkey"
            columns: ["user_id"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
        ]
      }
    }
    Views: {
      leaderboard: {
        Row: {
          current_xp: number | null
          display_name: string | null
          level: number | null
          pfp: string | null
          rank: number | null
          title: string | null
          total_xp: number | null
          user_id: string | null
          xp_required: number | null
        }
        Relationships: []
      }
      pg_stat_monitor: {
        Row: {
          application_name: string | null
          blk_read_time: number | null
          blk_write_time: number | null
          bucket: number | null
          bucket_done: boolean | null
          bucket_start_time: string | null
          calls: number | null
          client_ip: unknown | null
          cmd_type: number | null
          cmd_type_text: string | null
          comments: string | null
          cpu_sys_time: number | null
          cpu_user_time: number | null
          datname: string | null
          dbid: unknown | null
          elevel: number | null
          jit_emission_count: number | null
          jit_emission_time: number | null
          jit_functions: number | null
          jit_generation_time: number | null
          jit_inlining_count: number | null
          jit_inlining_time: number | null
          jit_optimization_count: number | null
          jit_optimization_time: number | null
          local_blks_dirtied: number | null
          local_blks_hit: number | null
          local_blks_read: number | null
          local_blks_written: number | null
          max_exec_time: number | null
          max_plan_time: number | null
          mean_exec_time: number | null
          mean_plan_time: number | null
          message: string | null
          min_exec_time: number | null
          min_plan_time: number | null
          pgsm_query_id: number | null
          planid: number | null
          plans: number | null
          query: string | null
          query_plan: string | null
          queryid: number | null
          relations: string[] | null
          resp_calls: string[] | null
          rows: number | null
          shared_blks_dirtied: number | null
          shared_blks_hit: number | null
          shared_blks_read: number | null
          shared_blks_written: number | null
          sqlcode: string | null
          stddev_exec_time: number | null
          stddev_plan_time: number | null
          temp_blk_read_time: number | null
          temp_blk_write_time: number | null
          temp_blks_read: number | null
          temp_blks_written: number | null
          top_query: string | null
          top_queryid: number | null
          toplevel: boolean | null
          total_exec_time: number | null
          total_plan_time: number | null
          userid: unknown | null
          username: string | null
          wal_bytes: number | null
          wal_fpi: number | null
          wal_records: number | null
        }
        Relationships: []
      }
      user_discord_invites: {
        Row: {
          amount: number | null
          guild_id: number | null
          type: number | null
          user_id: string | null
        }
        Relationships: []
      }
      user_levels: {
        Row: {
          current_xp: number | null
          display_name: string | null
          level: number | null
          pfp: string | null
          rank: number | null
          title: string | null
          total_xp: number | null
          user_id: string | null
          xp_required: number | null
        }
        Relationships: []
      }
      user_rewards_with_queue: {
        Row: {
          achievement_id: number | null
          archived_at: string | null
          created_at: string | null
          cutoff: string | null
          enqueued_at: string | null
          handled: boolean | null
          msg_id: number | null
          next_request_at: string | null
          queue_name: string | null
          requires_handling: boolean | null
          retryable: boolean | null
          reward_id: number | null
          status: string | null
          type: number | null
          user_id: string | null
        }
        Relationships: [
          {
            foreignKeyName: "user_rewards_achievement_id_user_id_fkey"
            columns: ["achievement_id", "user_id"]
            isOneToOne: false
            referencedRelation: "user_achievements"
            referencedColumns: ["achievement_id", "user_id"]
          },
          {
            foreignKeyName: "user_rewards_reward_id_fkey"
            columns: ["reward_id"]
            isOneToOne: false
            referencedRelation: "rewards"
            referencedColumns: ["id"]
          },
        ]
      }
    }
    Functions: {
      bulk_import_nfts: {
        Args: {
          p_collection_id: string
          p_provider: string
          nft_records: Json[]
        }
        Returns: Json
      }
      check_twitter_follow: {
        Args: { p_user_id: string; p_leader_id: number }
        Returns: boolean
      }
      decode_error_level: {
        Args: { elevel: number }
        Returns: string
      }
      get_and_update_twitter_leader: {
        Args: Record<PropertyKey, never>
        Returns: {
          twitter_id: string
          screen_name: string
          exit_on_duplicate: boolean
        }[]
      }
      get_cmd_type: {
        Args: { cmd_type: number }
        Returns: string
      }
      get_discord_member: {
        Args: { p_user_id: string; p_guild_id: number }
        Returns: {
          guild_id: number
          meta: Json
        }[]
      }
      get_github_contributions: {
        Args: { p_user_id: string }
        Returns: {
          owner: string
          repo: string
          meta: Json
        }[]
      }
      get_github_starred_repos: {
        Args: { p_user_id: string }
        Returns: {
          owner: string
          repo: string
        }[]
      }
      get_histogram_timings: {
        Args: Record<PropertyKey, never>
        Returns: string
      }
      get_next_user_batch: {
        Args:
          | { batch_size: number }
          | { batch_size: number; update_interval: unknown }
        Returns: {
          user_id: string
          mission_ids: number[]
        }[]
      }
      get_twitter_follows: {
        Args: { p_user_id: string }
        Returns: {
          leader_id: string
        }[]
      }
      get_unearned_achievements: {
        Args: { achievement_type: number } | { p_user_id: string }
        Returns: {
          user_id: string
          achievement_id: number
          meta: Json
        }[]
      }
      get_unearned_active_missions: {
        Args: { mission_type: number }
        Returns: {
          user_id: string
          mission_id: number
          type: number
          meta: Json
        }[]
      }
      get_unique_chains: {
        Args: { p_user_id: string }
        Returns: number
      }
      get_user_missions: {
        Args: { p_user_id: string }
        Returns: {
          user_id: string
          mission_ids: number[]
          ticked: boolean
        }[]
      }
      histogram: {
        Args: { _bucket: number; _quryid: number }
        Returns: Record<string, unknown>[]
      }
      insert_nfts_from_stargaze_json: {
        Args: { data: Json }
        Returns: Json
      }
      insert_points: {
        Args: {
          user_id: string
          partner_id: number
          reason: string
          points: number
          start_time: string
          end_time: string
        }
        Returns: undefined
      }
      pg_stat_monitor_internal: {
        Args: { showtext: boolean }
        Returns: Record<string, unknown>[]
      }
      pg_stat_monitor_reset: {
        Args: Record<PropertyKey, never>
        Returns: undefined
      }
      pg_stat_monitor_version: {
        Args: Record<PropertyKey, never>
        Returns: string
      }
      pgsm_create_11_view: {
        Args: Record<PropertyKey, never>
        Returns: number
      }
      pgsm_create_13_view: {
        Args: Record<PropertyKey, never>
        Returns: number
      }
      pgsm_create_14_view: {
        Args: Record<PropertyKey, never>
        Returns: number
      }
      pgsm_create_15_view: {
        Args: Record<PropertyKey, never>
        Returns: number
      }
      pgsm_create_17_view: {
        Args: Record<PropertyKey, never>
        Returns: number
      }
      pgsm_create_view: {
        Args: Record<PropertyKey, never>
        Returns: number
      }
      range: {
        Args: Record<PropertyKey, never>
        Returns: string[]
      }
      tick_user: {
        Args: { user_id_param: string }
        Returns: undefined
      }
      upsert_user_achievements: {
        Args: {
          p_user_id: string
          p_achievements: Json[]
          p_do_update?: boolean
        }
        Returns: {
          achievement_id: number
          achieved_at: string
          created_at: string
          progression: number
          threshold: number
        }[]
      }
    }
    Enums: {
      [_ in never]: never
    }
    CompositeTypes: {
      [_ in never]: never
    }
  }
}

type DefaultSchema = Database[Extract<keyof Database, "public">]

export type Tables<
  DefaultSchemaTableNameOrOptions extends
    | keyof (DefaultSchema["Tables"] & DefaultSchema["Views"])
    | { schema: keyof Database },
  TableName extends DefaultSchemaTableNameOrOptions extends {
    schema: keyof Database
  } ? keyof (
      & Database[DefaultSchemaTableNameOrOptions["schema"]]["Tables"]
      & Database[DefaultSchemaTableNameOrOptions["schema"]]["Views"]
    )
    : never = never,
> = DefaultSchemaTableNameOrOptions extends { schema: keyof Database } ? (
    & Database[DefaultSchemaTableNameOrOptions["schema"]]["Tables"]
    & Database[DefaultSchemaTableNameOrOptions["schema"]]["Views"]
  )[TableName] extends {
    Row: infer R
  } ? R
  : never
  : DefaultSchemaTableNameOrOptions extends keyof (
    & DefaultSchema["Tables"]
    & DefaultSchema["Views"]
  ) ? (
      & DefaultSchema["Tables"]
      & DefaultSchema["Views"]
    )[DefaultSchemaTableNameOrOptions] extends {
      Row: infer R
    } ? R
    : never
  : never

export type TablesInsert<
  DefaultSchemaTableNameOrOptions extends
    | keyof DefaultSchema["Tables"]
    | { schema: keyof Database },
  TableName extends DefaultSchemaTableNameOrOptions extends {
    schema: keyof Database
  } ? keyof Database[DefaultSchemaTableNameOrOptions["schema"]]["Tables"]
    : never = never,
> = DefaultSchemaTableNameOrOptions extends { schema: keyof Database }
  ? Database[DefaultSchemaTableNameOrOptions["schema"]]["Tables"][TableName] extends {
    Insert: infer I
  } ? I
  : never
  : DefaultSchemaTableNameOrOptions extends keyof DefaultSchema["Tables"]
    ? DefaultSchema["Tables"][DefaultSchemaTableNameOrOptions] extends {
      Insert: infer I
    } ? I
    : never
  : never

export type TablesUpdate<
  DefaultSchemaTableNameOrOptions extends
    | keyof DefaultSchema["Tables"]
    | { schema: keyof Database },
  TableName extends DefaultSchemaTableNameOrOptions extends {
    schema: keyof Database
  } ? keyof Database[DefaultSchemaTableNameOrOptions["schema"]]["Tables"]
    : never = never,
> = DefaultSchemaTableNameOrOptions extends { schema: keyof Database }
  ? Database[DefaultSchemaTableNameOrOptions["schema"]]["Tables"][TableName] extends {
    Update: infer U
  } ? U
  : never
  : DefaultSchemaTableNameOrOptions extends keyof DefaultSchema["Tables"]
    ? DefaultSchema["Tables"][DefaultSchemaTableNameOrOptions] extends {
      Update: infer U
    } ? U
    : never
  : never

export type Enums<
  DefaultSchemaEnumNameOrOptions extends
    | keyof DefaultSchema["Enums"]
    | { schema: keyof Database },
  EnumName extends DefaultSchemaEnumNameOrOptions extends {
    schema: keyof Database
  } ? keyof Database[DefaultSchemaEnumNameOrOptions["schema"]]["Enums"]
    : never = never,
> = DefaultSchemaEnumNameOrOptions extends { schema: keyof Database }
  ? Database[DefaultSchemaEnumNameOrOptions["schema"]]["Enums"][EnumName]
  : DefaultSchemaEnumNameOrOptions extends keyof DefaultSchema["Enums"]
    ? DefaultSchema["Enums"][DefaultSchemaEnumNameOrOptions]
  : never

export type CompositeTypes<
  PublicCompositeTypeNameOrOptions extends
    | keyof DefaultSchema["CompositeTypes"]
    | { schema: keyof Database },
  CompositeTypeName extends PublicCompositeTypeNameOrOptions extends {
    schema: keyof Database
  } ? keyof Database[PublicCompositeTypeNameOrOptions["schema"]]["CompositeTypes"]
    : never = never,
> = PublicCompositeTypeNameOrOptions extends { schema: keyof Database }
  ? Database[PublicCompositeTypeNameOrOptions["schema"]]["CompositeTypes"][CompositeTypeName]
  : PublicCompositeTypeNameOrOptions extends keyof DefaultSchema["CompositeTypes"]
    ? DefaultSchema["CompositeTypes"][PublicCompositeTypeNameOrOptions]
  : never

export const Constants = {
  public: {
    Enums: {},
  },
} as const

export type JoinedAchievement = Database["public"]["Tables"]["achievements"]["Row"] & {
  category?: {
    title: string
  } | null
  subcategory?: {
    title: string
  } | null
  reward_achievements?: {
    rewards: {
      title: string | null
      cutoff: string | null
    }[]
  }[]
}
