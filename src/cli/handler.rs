use std::collections::HashMap;
use std::time::Duration;

use uuid::Uuid;

use crate::cli::nodeCli::{NodeCommands, PeerCommands};
use crate::network::server::{get_connected_peers, PeerRegistry};
use crate::routing::bundleManager::BundleManager;
use crate::routing::engine::RoutingEngine;
use crate::routing::model::{Bundle, BundleKind, Node};
use crate::routing::scf::store;

pub struct CliContext {
    pub registry: PeerRegistry,
    pub engines: HashMap<Uuid, RoutingEngine>,
    pub managers: HashMap<Uuid, BundleManager>,
    pub retry_interval: Duration,
}

fn find_node<'a>(nodes: &'a [Node], name: &str) -> &'a Node {
    nodes.iter().find(|n| n.name == name).unwrap_or_else(|| {
        eprintln!("No node named '{}' found. Available nodes:", name);
        for n in nodes {
            eprintln!("  - {}", n.name);
        }
        std::process::exit(1);
    })
}

fn find_node_mut<'a>(nodes: &'a mut [Node], name: &str) -> &'a mut Node {
    nodes.iter_mut().find(|n| n.name == name).unwrap_or_else(|| {
        eprintln!("No node named '{}' found. Available nodes:", name);
        for n in nodes {
            eprintln!("  - {}", n.name);
        }
        std::process::exit(1);
    })
}

pub async fn handle_command(command: NodeCommands, nodes: &mut Vec<Node>, ctx: &mut CliContext) {
    match command {
        NodeCommands::All => {
            if nodes.is_empty() {
                println!("No nodes found.");
            } else {
                println!("Nodes in demo ({}):", nodes.len());
                for node in nodes.iter() {
                    println!(
                        "  - {} | {} | {}:{} | peers: {}",
                        node.name, node.id, node.address, node.port, node.peers.len()
                    );
                }
            }
        }

        NodeCommands::Start { name, server } => {
            let node = find_node(nodes, &name);
            println!("Starting node {} and registering with server {}...", node.name, server);
            println!("Node {} ({}) is ready.", node.name, node.id);
        }

        NodeCommands::Stop { name } => {
            let node = find_node(nodes, &name);
            println!("Stopping node {}...", node.name);
        }

        NodeCommands::Status { name } => {
            let node = find_node(nodes, &name);
            let stored = ctx.managers.get(&node.id).map(|m| m.all().len()).unwrap_or(0);
            println!("ID      : {}", node.id);
            println!("Name    : {}", node.name);
            println!("Address : {}:{}", node.address, node.port);
            println!("Peers   : {}", node.peers.len());
            println!("Bundles : {}", stored);
        }

        NodeCommands::Send { from, to, message, ttl } => {
            let src = find_node(nodes, &from).clone();
            let dst = find_node(nodes, &to).clone();

            let mut bundle = Bundle::new(
                src.clone(),
                dst,
                BundleKind::Data { msg: message },
                ttl,
            );

            let manager = ctx
                .managers
                .get_mut(&src.id)
                .expect("No BundleManager found for source node");

            store(&mut bundle, manager);

            let engine = ctx
                .engines
                .get(&src.id)
                .expect("No RoutingEngine found for source node");

            engine
                .route_bundle(&mut bundle, manager, ctx.retry_interval)
                .await;

            println!("Bundle {} routed from {}.", bundle.id, src.name);
        }

        NodeCommands::Peers { name, command } => {
            let node = find_node_mut(nodes, &name);
            handle_peer_command(command, node, &ctx.registry);
        }

        #[cfg(feature = "debug")]
        NodeCommands::Debug { name } => match name {
            Some(name) => {
                let node = find_node(nodes, &name);
                println!("{}", serde_json::to_string_pretty(node).unwrap());
            }
            None => {
                println!("{}", serde_json::to_string_pretty(nodes).unwrap());
            }
        },
    }
}

fn handle_peer_command(command: PeerCommands, node: &mut Node, registry: &PeerRegistry) {
    match command {
        PeerCommands::ListPeers => {
            if node.peers.is_empty() {
                println!("No known peers for {}.", node.name);
            } else {
                println!("Peers for {}:", node.name);
                for peer in &node.peers {
                    println!("  - {}", peer);
                }
            }
        }

        PeerCommands::GetConnectedPeers { ids } => {
            let uuids: Vec<Uuid> = ids
                .iter()
                .map(|s| Uuid::parse_str(s).expect("Invalid UUID"))
                .collect();

            let peers = get_connected_peers(registry, &uuids);
            println!("Connected peers found: {}", peers.len());
            for p in peers {
                println!(
                    "  - {} | {} | {}:{}",
                    p.node.name, p.node.id, p.node.address, p.node.port
                );
            }
        }

        PeerCommands::Add { id } => {
            let uuid = Uuid::parse_str(&id).expect("Invalid UUID");
            if node.peers.contains(&uuid) {
                println!("{} already knows peer {}.", node.name, uuid);
            } else {
                node.peers.push(uuid);
                println!("Peer {} added to {}.", uuid, node.name);
            }
        }

        PeerCommands::Remove { id } => {
            let uuid = Uuid::parse_str(&id).expect("Invalid UUID");
            let before = node.peers.len();
            node.peers.retain(|p| *p != uuid);
            if node.peers.len() < before {
                println!("Peer {} removed from {}.", uuid, node.name);
            } else {
                println!("Peer {} was not in {} peer list.", uuid, node.name);
            }
        }
    }
}