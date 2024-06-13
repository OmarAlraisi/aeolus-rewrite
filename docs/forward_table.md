# <span style="font-size: 28px"><span style="text-transform: uppercase;"><span style="font-size: 36px">a</span>eolus</span> - Forward Tables & Hashing</span>

**<i>Aeolus</i>** uses a simple two-array structure for the [`forwarding table`](https://www.baeldung.com/cs/routing-vs-forwarding-tables#:~:text=A%20forwarding%20table%20simply%20forwards,%2C%20and%20host%2Dspecific%20methods.), and uses a hash of the [`4-tuple`](https://www.cse.iitb.ac.in/~cs348m/notes/lec08.txt#:~:text=TCP%20uses%204%2Dtuple%20(source%20IP%2C%20source%0A%20%20port%2C%20destination%20IP%2C%20destination%20port)) to index the table.

## Forward Table

When a new packet arrives, we generate a [`hash`](#hashing) key which is used to retrieve the destination server from the forwarding table.

Since **<i>Aeolus</i>** allows for adding and removing nodes from the cluster, nodes can be in `ACTIVE`, `DRAINING`, or `FILLING` states. The design of the forwarding table for **<i>Aeolus</i>** was inspired by [`GLB Director Hashing (glb)`](https://github.com/github/glb-director/blob/master/docs/development/glb-hashing.md), however, unlike **<i>glb</i>**, one of the key considerations for **<i>Aeolus</i>** was that we wanted to be able to drain more than one server at a time (i.e. in `DRAINING` state), which **<i>glb</i>** didn't allow, or add more than one server at a time (i.e. in `FILLING` state). - **_The caveat is that you can not have servers in the_ `DRAINGIN` _state and others in the `FILLING` state at the same time._**

The forwarding table consists of two arrays, representing the first hop and second hop destinations. If we have `s` servers (`0 to s-1`), each array will consist of `s * floor(s / 2)` entries, where each server will appear `floor(s / 2)` times in the array.

### Removing Nodes From the Cluster

---

Assuming no server currently in the `DRAINIG` state, when we move `x` to the `DRAINING` state, it becomes an <i>**always-second hop**</i> server. To remove the server `x` from the first hop array we perform the following:

- Split the servers into two groups, one with servers who have even indices in an array of all running servers, and the other with for servers with odd indices.

- Iterate over all entries of the first hop array and replace all occurrences of server `x` with the servers in group that doesn't contain `x`.

Once `x` is drained, remove all occurrences of `x` in the second hop array with the server at the same index in the first hop array.

However, if there is at least one server `y` that is in the `DRAINING` state, then `x` is ONLY allowed to be moved to the `DRAINING` state if it belongs to the same group of `y`.

#### Example:

Assume we have `7` servers, the forwarding table arrays will have `7 * floor(7 / 2) = 21` entries. When all servers are in the `ACTIVE` state, the forwarding table will look as such:

```Text
first  = [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 6]
second = [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 6]
```

Now, let's say we want to remove server `4` from the cluster.

```Text
Step I: Split the servers into groups.
    group_a = [0, 2, 4, 6]
    group_b = [1, 3, 5]

Step II: Replace all occurrences of server `4` in the first hop array.
    first  = [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 1, 3, 5, 5, 5, 5, 6, 6, 6]
    second = [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 6]

**Note: At this point, while server `4` is in the `DRAINING` state, you can NOT 
        move servers, `1`, `3`, or `5` to the `DRAINING` state, since they are 
        the first destination for packets that are meant for server `4`. You 
        can only chage the states of servers `0`, `2`, and `6` to the `DRAINING`
        state.

Once server `4` is fully drained, remove all occurrences of server `4` from the
second hop array.
    first  = [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 1, 3, 5, 5, 5, 5, 6, 6, 6]
    second = [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 1, 3, 5, 5, 5, 5, 6, 6, 6]
```

Now, that there is no server in the `DRAINING` state, you will have to recreate `group_a` and `group_b` once you start draining a new server. They will look like:

```Text
    group_a = [0, 2, 5]
    group_a = [1, 3, 6]
```

### Adding Nodes to the Cluster

---

Adding nodes to Aeolus is more straightforward, all that needs to be done is adding the node to the first hop array without modifying the second hop array. And to ensure mainitaining a good distribution when adding a server `x`:

- Calculate the appropriate frequency of `x` in the forwarding table arrays.
- Track the nodes with the most frequency. - `MaxHeap`.
    - Move them from the first hop array to the same position in the second hop array.
    - Substite them in the first hop array with `x`.

TODO: Find out a way to add more than one server at once.

## Hashing

TODO: The below hash doesn't really distribute the load eaqually, there's a very high chance of having most of the requests to hit the same server.

`hash_key = (src_ip | dest_ip | (src_port as u32) | (dest_port as u32)) % (n * (n - 1))`

Assume we have a packet with the following info:

| Key       | Value       |
| --------- | ----------- |
| src_ip    | 203.0.113.1 |
| src_port  | 1234        |
| dest_ip   | 203.0.113.2 |
| dest_port | 4321        |
```
hash_key = (3405803777 | 3405803778 | (1234 as u32) | (4321 as u32)) % (5 * (5 - 1))
hash_key = (3405805043) % 20
hash_key = 3
```

Therefore this packet gets forwarded to server `4` for the first hop, if no established connection, then gets forwarded to server `0`.
