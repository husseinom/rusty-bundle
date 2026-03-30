use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "node")]

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

enum NodeCommands {
    /// Start the node and register it with the server
    Start {
        /// IP address this node listens on
        #[arg(short, long)]
        address: String,

        /// Port this node listens on
        #[arg(short, long)]
        port: u16,

        /// Server address to register with (ip:port)
        #[arg(short, long)]
        server: String,
    },
    /// Stop the node server
    Stop,
    
    /// Get the status of the node
    Status,

    /// Manage known peers
    Peers {
        #[command(subcommand)]
        command: PeerCommands,
    },

    Send {
        /// name of the destination node
        #[arg(long)]
        to: String,

        /// The message content
        #[arg(long)]
        message: String,

        /// Time-to-live in seconds
        #[arg(long)]
        ttl: u64,
    },

}

#[derive(Subcommand)]
pub enum PeerCommands {
    /// List all known peers
    ListPeers,

    /// Fetch one or more peers from the server by UUID
    GetConnectedPeers {
        /// One or more peer UUIDs
        #[arg(required = true, num_args = 1..)]
        ids: Vec<String>,
    },

    /// Add a peer UUID to your local peer list
    Add {
        /// UUID of the peer to add
        id: String,
    },

    /// Remove a peer UUID from your local peer list
    Remove {
        /// UUID of the peer to remove
        id: String,
    },
}

