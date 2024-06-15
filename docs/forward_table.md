# <span style="font-size: 28px"><span style="text-transform: uppercase;"><span style="font-size: 36px">a</span>eolus</span> - Forward Tables & Hashing</span>

**<i>Aeolus</i>** uses a simple two-array structure for the [`forwarding table`](https://www.baeldung.com/cs/routing-vs-forwarding-tables#:~:text=A%20forwarding%20table%20simply%20forwards,%2C%20and%20host%2Dspecific%20methods.), and uses a hash of the [`4-tuple`](https://www.cse.iitb.ac.in/~cs348m/notes/lec08.txt#:~:text=TCP%20uses%204%2Dtuple%20(source%20IP%2C%20source%0A%20%20port%2C%20destination%20IP%2C%20destination%20port)) to index the table.

## Forward Table

When a new packet arrives, we generate a [`hash`](#hashing) key which is used to retrieve the destination server from the forwarding table.

Since **<i>Aeolus</i>** allows for adding and removing nodes from the cluster, nodes can be in **`ACTIVE`**, **`DRAINING`**, **`FILLING`**, or **`INACTIVE`** states. The design of the forwarding table for **<i>Aeolus</i>** was inspired by [`GLB Director Hashing (glb)`](https://github.com/github/glb-director/blob/master/docs/development/glb-hashing.md), however, unlike **<i>glb</i>**, one of the key considerations for **<i>Aeolus</i>** was that we wanted to be able to drain more than one server at a time (i.e. in **`DRAINING`** state), which **<i>glb</i>** didn't allow, or add more than one server at a time (i.e. in **`FILLING`** state). - **_The caveat is that you can not have servers in the_ `DRAINGIN` _state and others in the `FILLING` state at the same time._**

The forwarding table consists of two arrays, representing the first hop and second hop destinations. If we have `s` servers (`0 to s-1`), each array will consist of `s * floor(s / 2)` entries, where each server will appear `floor(s / 2)` times in the array.

### Removing Nodes From the Cluster
---

Assuming no server currently in the **`DRAINIG`** state, when we move `x` to the **`DRAINING`** state, it becomes an <i>**always-second hop**</i> server. To remove the server `x` from the first hop array we perform the following:

- Split the servers into two groups, one with servers who have even indices in an array of all running servers, and the other with for servers with odd indices.

- Iterate over all entries of the first hop array and replace all occurrences of server `x` with the servers in group that doesn't contain `x`.

Once `x` is drained, remove all occurrences of `x` in the second hop array with the server at the same index in the first hop array.

However, if there is at least one server `y` that is in the **`DRAINING`** state, then `x` is ONLY allowed to be moved to the **`DRAINING`** state if it belongs to the same group of `y`.

<h4 id="removing-example">Example:</h4>

Assume we have `7` servers, the forwarding table arrays will have `7 * floor(7 / 2) = 21` entries. When all servers are in the **`ACTIVE`** state, the forwarding table will look as such:

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
        state. - For the sake of the example, server `2` will also be removed.

Once servers `2` and `4` are fully drained, remove all theirs occurrences from
the second hop array.
    first  = [0, 0, 0, 1, 1, 1, 1, 3, 5, 3, 3, 3, 1, 3, 5, 5, 5, 5, 6, 6, 6]
    second = [0, 0, 0, 1, 1, 1, 1, 3, 5, 3, 3, 3, 1, 3, 5, 5, 5, 5, 6, 6, 6]
```

Now, that there is no server in the **`DRAINING`** state, you will have to recreate `group_a` and `group_b` once you start draining a new server. They will look like:

```Text
    group_a = [0, 2, 5]
    group_a = [1, 3, 6]
```

### Adding Nodes to the Cluster
---

Adding nodes to Aeolus is more straightforward, all that needs to be done is adding the node to the first hop array without modifying the second hop array. And to ensure mainitaining a good distribution when adding a server `x`:

- Calculate the appropriate frequency of `x` in the forwarding table arrays.
- Track the nodes, _**that are in the `ACTIVE` state**_, with the most frequency. - `MaxHeap`.
    - Move them from the first hop array to the same position in the second hop array.
    - Substite them in the first hop array with `x`.
- Once server `x` is not redirecting anymore packets, chage its state to **`ACTIVE`**

<h4 id="appropriate-frequency">Filling Node Frequency:</h4>

Let's say we have `num_of_entries` entry in each array of the forwarding table, and we currently have `num_of_nodes` nodes that are either in the **`ACTIVE`** or the **`FILLING`** states. It might be intuitive to add `(num_of_entries / num_of_nodes)` of the node `x` to the forwarding table. However, this will make it difficult to add more new nodes to the cluster, as nodes that are in the **`FILLING`** state can not be moved to the second hop array.

To address this, Aeolus uses the following equation to get the appropriate frequency for `x`:

```Text
frequency = ceil((floor(num_of_entries / num_of_nodes) / 3) * 2)
```

_**This equation basically mean we add two thirds of the result of the intuitive approach mentioned above.**_

<h4 id="adding-example">Example:</h4>

Continuing our previous example, the forwarding table will start in the following state:

```Text
first  = [0, 0, 0, 1, 1, 1, 1, 3, 5, 3, 3, 3, 1, 3, 5, 5, 5, 5, 6, 6, 6]
second = [0, 0, 0, 1, 1, 1, 1, 3, 5, 3, 3, 3, 1, 3, 5, 5, 5, 5, 6, 6, 6]
```

Let's add server `4` back into the cluster.

```Text
Step I: Calculate the desired frequency for server `4`.
    freq = ceil((floor(num_of_entries / num_of_active) / 3) * 2)
    freq = ceil((floor(21 / 5) / 3) * 2) = 3
    freq = ceil((floor(4.2) / 3) * 2) = 3
    freq = ceil((4 / 2) * 3) = 3
    freq = ceil(2.66666667) = 3

first  = [0, 0, 0, 1, 1, 1, 1, 3, 5, 3, 3, 3, 1, 3, 5, 5, 5, 5, 6, 6, 6]
second = [0, 0, 0, 1, 1, 1, 1, 3, 5, 3, 3, 3, 1, 3, 5, 5, 5, 5, 6, 6, 6]
```

TODO: Continue on the same [example](#removing-example) of the previous section.

### Node State
---

A node can be in one of four states:

- **`ACTIVE`**: The node is functioning normally.
- **`DRAINING`**: The node is being drained to be taken out of the cluster, however, it might still have some established connections.
- **`FILLING`**: The nodes can be treated as if it is in the **`ACTIVE`** state, but it might be dangerous to change its states, as its second hops might still be in **`ACTIVE`** or **`DRAINING`** state.
- **`INACTIVE`**: The node is not running, and can be changed from the second hop array at any time.

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
