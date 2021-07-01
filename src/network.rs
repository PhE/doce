use std::sync::Mutex;

use bevy::{prelude::*, tasks::IoTaskPool};
use futures::{executor::block_on, prelude::*, select};
use libp2p::{
    development_transport,
    gossipsub::{
        error::GossipsubHandlerError, Gossipsub, GossipsubConfigBuilder, GossipsubEvent,
        IdentTopic, MessageAuthenticity,
    },
    identity::Keypair,
    swarm::SwarmEvent,
    Multiaddr, PeerId, Swarm,
};

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<NetworkManager>()
            .add_event::<NetworkEvent>()
            .add_system_to_stage(CoreStage::First, update_network.system());
    }
}

pub struct NetworkManager {
    command_tx: libp2p::futures::channel::mpsc::UnboundedSender<NetworkCommand>,
    event_rx: Mutex<std::sync::mpsc::Receiver<NetworkEvent>>,
}

impl NetworkManager {
    pub fn dial(&mut self, addr: NetworkAddress) {
        self.command_tx
            .unbounded_send(NetworkCommand::Dial(addr))
            .unwrap();
    }

    pub fn subscribe(&mut self, topic: NetworkTopic) {
        self.command_tx
            .unbounded_send(NetworkCommand::Subscribe(topic))
            .unwrap();
    }

    pub fn unsubscribe(&mut self, topic: NetworkTopic) {
        self.command_tx
            .unbounded_send(NetworkCommand::Unsubscribe(topic))
            .unwrap();
    }

    pub fn publish(&mut self, topic: NetworkTopic, data: impl Into<Vec<u8>>) {
        self.command_tx
            .unbounded_send(NetworkCommand::Publish(topic, data.into()))
            .unwrap();
    }
}

impl FromWorld for NetworkManager {
    fn from_world(world: &mut World) -> Self {
        let (command_tx, mut command_rx) =
            libp2p::futures::channel::mpsc::unbounded::<NetworkCommand>();
        let (event_tx, event_rx) =
            std::sync::mpsc::channel::<SwarmEvent<GossipsubEvent, GossipsubHandlerError>>();
        let event_rx = Mutex::new(event_rx);

        let io_task_pool = world.get_resource::<IoTaskPool>().unwrap();
        io_task_pool
            .spawn(async move {
                let mut swarm = create_network_swarm();
                    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();

                loop {
                    select! {
                        command = command_rx.select_next_some() => {
                            match command {
                                NetworkCommand::Dial(addr) => swarm.dial_addr(addr).unwrap(),
                                NetworkCommand::Subscribe(topic) => swarm.behaviour_mut().subscribe(&topic).map(|_| ()).unwrap(),
                                NetworkCommand::Unsubscribe(topic) => swarm.behaviour_mut().unsubscribe(&topic).map(|_| ()).unwrap(),
                                NetworkCommand::Publish(topic, data) => swarm.behaviour_mut().publish(topic, data).map(|_| ()).unwrap(),
                            };
                        }
                        event = swarm.next_event().fuse() => event_tx.send(event).unwrap(),
                    }
                }
            })
            .detach();

        Self {
            command_tx,
            event_rx,
        }
    }
}

pub type NetworkAddress = Multiaddr;
pub type NetworkEvent = SwarmEvent<GossipsubEvent, GossipsubHandlerError>;
pub type NetworkTopic = IdentTopic;

enum NetworkCommand {
    Dial(NetworkAddress),
    Subscribe(NetworkTopic),
    Unsubscribe(NetworkTopic),
    Publish(NetworkTopic, Vec<u8>),
}

fn create_network_swarm() -> Swarm<Gossipsub> {
    let local_key = Keypair::generate_ed25519();
    let local_peer_id = PeerId::from_public_key(local_key.public());
    let transport = block_on(development_transport(local_key.clone())).unwrap();
    let gossipsub_config = GossipsubConfigBuilder::default().build().unwrap();
    let gossipsub: Gossipsub =
        Gossipsub::new(MessageAuthenticity::Signed(local_key), gossipsub_config).unwrap();

    Swarm::new(transport, gossipsub, local_peer_id)
}

fn update_network(manager: Res<NetworkManager>, mut event_writer: EventWriter<NetworkEvent>) {
    for event in manager.event_rx.lock().unwrap().try_iter() {
        info!("{:?}", event);
        event_writer.send(event);
    }
}
