# p2psocial

Description: First (?) decentalised social media
Principal: All components (account creation, sharing, moderation) are not controlled by a single authority.
Theory: Inspired by real social networking groups, peers are treated as "friends", where "friends of a friend" are also likely to be friends.

# Protocol

1. A node joins the network by connecting to bootstrap nodes
2. They request for posts and score each peer's response
3. Peers that are statistically significant are requested to share their secondary peers
4. Peers only share secondary peers if the requesting peer is also (somewhat) statistically significant.
4. The node attempts to maintain more than 8 nodes, less than 32.

# Scoring

Each node has an internal elo score (default=1200)
When the node likes another post, they 'lose'
When the node dislikes another post, they 'win'

These elo scores are private and untrusted, and are only used for internel reference of node quality.
Once a peer is statistically significant, they are either dropped or requested for secondary peers, while also trying to keep within the [8, 32] bounds.



# Potential Issues


## Issue
A malicious node could constantly spam secondary peer requests (ignoring the bounds), then send malicous posts.
## Solution 
Nodes only accept a peer request only if the requesting node is better than the average peer (ie, more than a 50% chance of good peers).


## Issue
Bootstrap nodes reject all nodes due to scoring mechanism
## Solution
Bootstrap nodes must always accept peer requests


# Structs (Simple, unauthed)

Post {
    title: String
    ipfs_hash: String
}

PeerRequest {
    
}

PeerResponse {
    ids: Vec<NodeID>
}






