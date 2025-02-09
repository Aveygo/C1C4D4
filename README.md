# p2psocial


## A little rant

Hi, I'm a developer. This means I solve problems. Specifically computer problems. When people *say* they solved 'decentralised social media', what they actually do is create a ~~centralised~~ *federated* platform, and call it a day (mastodom, lemmy, bluesky, and a thousand others).  Even blockchain solutions like Odysee (rip libry), Dscvr, or OpenChat are imperfect - locking features behind a paywall and expecting people to pay for the ability to share their valuable speech. Even worse is that there are hundreds of thousands of platforms that all use 'web3 technology' (no seriously, just look on github), but the users of the platform are unable to control the network! The closest we got is with Scuttlebutt, which still relies on self hosted rooms (although for good reason I would add). Instead, we need something as simple as possible, but self moderating. Secure, but scalable. Something that is imperfect, but *truely* decentralised. A social media platform for the people, by the people, that the people run.

This is my solution in a bucket of a thousand (a true 927 moment), but I hope that it serves it's purpose and grows into something I could be proud of, or to at least inspire a better solution.


# Protocol

1. Alice joins the network by connecting to bootstrap nodes.
2. Alice requests for Bob's posts (who recursively does the same) and scores Bob on each one.
3. If Alice believes that Bob is an acceptable peer, she will request for Bob's peers.
4. Bob performs the request if Alice is similarly acceptable. 
4. The process repeats where Alice eventually finds her clostest peers. 

TLDR: A friend of my friend is also my friend

By restricting which users connect to who, each node becomes a moderator for their peers. Unfortunately this does mean that everyone has a role to play in managing the worst of humanity, but by building the network and only connecting to trusted peers, this issue can be solved. 


# Expected structure

Users would aggregate into their own networks to better align with their preferences. I predict that most networks would be around very broad topics like 'memes' or 'art', kind of like subreddits. When a universally popular post is submitted (eg news), it would 'fly' across these networks, first around the source of the post, then last to those that are the least likely to be interested. I would expect that most nodes would be seperated by ~8 degrees (similar to real life), with around 16 close peers that they query from. I would also expect that content creators would also collagulate together into tight networks as it would help prevent spam and reduce latency for collaboration purposes.

One potential issue is that the network is expected to be fully connected, so people that would want an otherwise private network may have their content leaked. In this instance, they should run their own bootstrap node. Thus each network has a 'network id' (the public key of the bootstrap node), which also acts as an insecure invite code (would be leaked in the DHT). The opposite extreme is also true, where a popular node (eg, a famous celebrity) might not have the bandwidth to connect to all their supporters. In this instance, a natural tree like structure should appear, where only the most die-hard fans are placed higher.

# Structs (Simplified, completely anonymous)

## Posts
```
PostsRequest {
    epoch: int
}
```

```
Post {
    title: String
    ipfs_hash: String
    epoch: int
}
```

```
PostsResponse {
    posts: Vec<Post>
}
```

## Peers
```
PeersRequest {
    epoch: int
}
```

```
PeersResponse {
    ids: Vec<NodeID>
    epoch: int
}
```




