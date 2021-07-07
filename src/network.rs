use std::{
    collections::VecDeque,
    sync::mpsc as sync_mpsc,
    sync::Mutex,
    task::{Context, Poll},
};

use bevy::{prelude::*, tasks::IoTaskPool};
use futures::{
    channel::mpsc as async_mpsc, channel::oneshot, executor::block_on, future::poll_fn, prelude::*,
    select,
};
pub use libp2p::{core::connection::ListenerId, PeerId, TransportError};
use libp2p::{
    development_transport,
    gossipsub::{
        error::PublishError, Gossipsub, GossipsubConfigBuilder, GossipsubEvent, IdentTopic,
        MessageAuthenticity,
    },
    identity::Keypair,
    swarm::{
        IntoProtocolsHandler, NetworkBehaviourEventProcess, PollParameters, ProtocolsHandler,
        SwarmEvent,
    },
    Multiaddr, Swarm,
};

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<NetworkManager>()
            .add_event::<NetworkEvent>()
            .add_system_to_stage(CoreStage::First, update_network.system());
    }
}

#[derive(libp2p::NetworkBehaviour)]
#[behaviour(out_event = "NetworkBehaviourEvent", poll_method = "poll")]
pub struct NetworkBehaviour {
    pub gossipsub: Gossipsub,
    #[behaviour(ignore)]
    events: VecDeque<NetworkBehaviourAction>,
}

impl NetworkBehaviour {
    fn poll(
        &mut self,
        _: &mut Context<'_>,
        _: &mut impl PollParameters,
    ) -> Poll<NetworkBehaviourAction> {
        if let Some(event) = self.events.pop_front() {
            return Poll::Ready(event);
        }

        Poll::Pending
    }
}

impl NetworkBehaviourEventProcess<GossipsubEvent> for NetworkBehaviour {
    fn inject_event(&mut self, event: GossipsubEvent) {
        self.events.push_back(NetworkBehaviourAction::GenerateEvent(
            NetworkBehaviourEvent::Gossipsub(event),
        ));
    }
}

#[derive(Debug)]
pub enum NetworkBehaviourEvent {
    Gossipsub(GossipsubEvent),
}

pub struct NetworkManager {
    local_peer_id: PeerId,
    command_tx: async_mpsc::UnboundedSender<NetworkCommand>,
    event_rx: Mutex<sync_mpsc::Receiver<NetworkEvent>>,
}

impl NetworkManager {
    pub fn local_peer_id(&self) -> PeerId {
        self.local_peer_id
    }

    pub fn listen_on(
        &mut self,
        addr: NetworkAddress,
    ) -> Result<ListenerId, TransportError<std::io::Error>> {
        let (result_tx, result_rx) = oneshot::channel();

        self.command_tx
            .unbounded_send(NetworkCommand::ListenOn(addr, result_tx))
            .unwrap();

        block_on(result_rx).unwrap()
    }

    pub fn remove_listener(&mut self, listener_id: ListenerId) {
        self.command_tx
            .unbounded_send(NetworkCommand::RemoveListener(listener_id))
            .unwrap();
    }

    pub fn dial_addr(&mut self, addr: NetworkAddress) {
        self.command_tx
            .unbounded_send(NetworkCommand::DialAddr(addr))
            .unwrap();
    }

    pub fn dial_peer(&mut self, peer_id: PeerId) {
        self.command_tx
            .unbounded_send(NetworkCommand::DialPeer(peer_id))
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
        let local_key = Keypair::generate_ed25519();
        let local_peer_id = PeerId::from_public_key(local_key.public());
        let (command_tx, mut command_rx) = async_mpsc::unbounded::<NetworkCommand>();
        let (event_tx, event_rx) = sync_mpsc::channel::<NetworkEvent>();
        let event_rx = Mutex::new(event_rx);

        let io_task_pool = world.get_resource::<IoTaskPool>().unwrap();
        io_task_pool
            .spawn(async move {
                let mut swarm = create_network_swarm(local_key, local_peer_id);

                loop {
                    select! {
                        command = command_rx.select_next_some() => handle_network_command(&mut swarm, command),
                        event = swarm.next_event().fuse() => {
                            // handle_network_event(&mut swarm, &event);
                            event_tx.send(event).unwrap();
                        }
                    }
                }
            })
            .detach();

        block_on(poll_fn(|context| command_tx.poll_ready(context))).unwrap();

        Self {
            local_peer_id,
            command_tx,
            event_rx,
        }
    }
}

pub type NetworkAddress = Multiaddr;
pub type NetworkEvent =
    SwarmEvent<
        <NetworkBehaviour as libp2p::swarm::NetworkBehaviour>::OutEvent,
        <<<NetworkBehaviour as libp2p::swarm::NetworkBehaviour>::ProtocolsHandler as IntoProtocolsHandler>::Handler as ProtocolsHandler>::Error
    >;
pub type NetworkTopic = IdentTopic;
type NetworkBehaviourAction = libp2p::swarm::NetworkBehaviourAction<
    <<<NetworkBehaviour as libp2p::swarm::NetworkBehaviour>::ProtocolsHandler as IntoProtocolsHandler>::Handler as ProtocolsHandler>::InEvent,
    <NetworkBehaviour as libp2p::swarm::NetworkBehaviour>::OutEvent
>;

enum NetworkCommand {
    ListenOn(
        NetworkAddress,
        oneshot::Sender<Result<ListenerId, TransportError<std::io::Error>>>,
    ),
    RemoveListener(ListenerId),
    DialAddr(NetworkAddress),
    DialPeer(PeerId),
    Subscribe(NetworkTopic),
    Unsubscribe(NetworkTopic),
    Publish(NetworkTopic, Vec<u8>),
}

fn create_network_swarm(local_key: Keypair, local_peer_id: PeerId) -> Swarm<NetworkBehaviour> {
    let transport = block_on(development_transport(local_key.clone())).unwrap();
    let gossipsub_config = GossipsubConfigBuilder::default().build().unwrap();
    let gossipsub: Gossipsub =
        Gossipsub::new(MessageAuthenticity::Signed(local_key), gossipsub_config).unwrap();
    let behaviour = NetworkBehaviour {
        gossipsub,
        events: VecDeque::new(),
    };

    Swarm::new(transport, behaviour, local_peer_id)
}

fn handle_network_command(swarm: &mut Swarm<NetworkBehaviour>, command: NetworkCommand) {
    match command {
        NetworkCommand::ListenOn(addr, sender) => sender.send(swarm.listen_on(addr)).unwrap(),
        NetworkCommand::RemoveListener(listener_id) => swarm.remove_listener(listener_id).unwrap(),
        NetworkCommand::DialAddr(addr) => swarm.dial_addr(addr).unwrap(),
        NetworkCommand::DialPeer(peer_id) => swarm.dial(&peer_id).unwrap(),
        NetworkCommand::Subscribe(topic) => swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&topic)
            .map(|_| ())
            .unwrap(),
        NetworkCommand::Unsubscribe(topic) => swarm
            .behaviour_mut()
            .gossipsub
            .unsubscribe(&topic)
            .map(|_| ())
            .unwrap(),
        // TODO: Properly return result to caller
        NetworkCommand::Publish(topic, data) => {
            match swarm.behaviour_mut().gossipsub.publish(topic, data) {
                Ok(_) => (),
                Err(PublishError::InsufficientPeers) => (),
                Err(error) => Err(error).unwrap(),
            }
        }
    };
}

fn update_network(manager: Res<NetworkManager>, mut event_writer: EventWriter<NetworkEvent>) {
    for event in manager.event_rx.lock().unwrap().try_iter() {
        info!("{:?}", event);
        event_writer.send(event);
    }
}
