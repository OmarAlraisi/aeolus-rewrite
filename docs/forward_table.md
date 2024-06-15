# <span style="font-size: 28px"><span style="text-transform: uppercase;"><span style="font-size: 36px">a</span>eolus</span> - Forwarding Tables & Hashing</span>

**<i>Aeolus</i>** uses a simple two-array structure for the [`forwarding table`](https://www.baeldung.com/cs/routing-vs-forwarding-tables#:~:text=A%20forwarding%20table%20simply%20forwards,%2C%20and%20host%2Dspecific%20methods.), and uses a hash of the [`4-tuple`](https://www.cse.iitb.ac.in/~cs348m/notes/lec08.txt#:~:text=TCP%20uses%204%2Dtuple%20(source%20IP%2C%20source%0A%20%20port%2C%20destination%20IP%2C%20destination%20port)) to index the table.

## Forwarding Table

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

_**Note that there is no need to copy server `x` to the second hop array once it state changes to `ACTIVE`.**_

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
    freq = ceil((floor(21 / 5) / 3) * 2) = 3

Step II: Track most frequent nodes and start adding server `4`.
    Loop i: MaxHeap.pop() => 1
        Replace 1 occurrence of server `1` with `4`.
        first  = [0, 0, 0, 4, 1, 1, 1, 3, 5, 3, 3, 3, 1, 3, 5, 5, 5, 5, 6, 6, 6]
        second = [0, 0, 0, 1, 1, 1, 1, 3, 5, 3, 3, 3, 1, 3, 5, 5, 5, 5, 6, 6, 6]

    Loop ii: MaxHeap.pop() => 3
        Replace 1 occurrence of server `3` with `4`.
        first  = [0, 0, 0, 4, 1, 1, 1, 4, 5, 3, 3, 3, 1, 3, 5, 5, 5, 5, 6, 6, 6]
        second = [0, 0, 0, 1, 1, 1, 1, 3, 5, 3, 3, 3, 1, 3, 5, 5, 5, 5, 6, 6, 6]

    Loop i: MaxHeap.pop() => 5
        Replace 1 occurrence of server `5` with `4`.
        first  = [0, 0, 0, 4, 1, 1, 1, 4, 4, 3, 3, 3, 1, 3, 5, 5, 5, 5, 6, 6, 6]
        second = [0, 0, 0, 1, 1, 1, 1, 3, 5, 3, 3, 3, 1, 3, 5, 5, 5, 5, 6, 6, 6]

**Note: We can also add server `2` back to the cluster, but we can never add it
        in the place of of server `4` (i.e. a server in the `FILLING` state).

Step III: Change the state of server `4` once it is not directing any connection.
    Note that we don't need to copy anything to the second hop array. And this 
    would be the state of the forwarding table when all servers are in the
    `ACTIVE` state:
        first  = [0, 0, 0, 4, 1, 1, 1, 4, 4, 3, 3, 3, 1, 3, 5, 5, 5, 5, 6, 6, 6]
        second = [0, 0, 0, 1, 1, 1, 1, 3, 5, 3, 3, 3, 1, 3, 5, 5, 5, 5, 6, 6, 6]
```

### Node State
---

A node can be in one of four states:

- **`ACTIVE`**: The node is functioning normally.
- **`DRAINING`**: The node is being drained to be taken out of the cluster, however, it might still have some established connections.
- **`FILLING`**: The nodes can be treated as if it is in the **`ACTIVE`** state, but it might be dangerous to change its states, as its second hops might still be in **`ACTIVE`** or **`DRAINING`** state.
- **`INACTIVE`**: The node is not running, and can be changed from the second hop array at any time.

## Hashing

Aeolus's hashing function is inspired by the Linux kernel's [**`inet_ehashfn`**](https://github.com/torvalds/linux/blob/a3e18a540541325a8c8848171f71e0d45ad30b2c/net/ipv4/inet_hashtables.c#L32); it takes the **`4-tuple`** and retreives a hash key to index the [**`forwarding table`**](#forwarding-table).

```Text
hash_key =
    (src_ip ^
    dst_ip ^
    (src_port << 16) ^
    src_port ^
    (dst_port << 8) ^
    src_port) % num_of_entries
```

<h4 id="hashing-example">Example:</h4>

Assume the cluster's forwarding table is in the following state:

```Text
first  = [6, 0, 0, 1, 1, 1, 1, 3, 5, 3, 3, 3, 1, 3, 5, 5, 5, 5, 6, 6, 0]
second = [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 6]
```

And we recieve a packet with the following info:

| Key       | Value       | Decimal    |
| --------- | ----------- | ---------- |
| src_ip    | 203.0.113.1 | 3405803777 |
| src_port  | 1234        | 1234       |
| dest_ip   | 203.0.113.2 | 3405803778 |
| dest_port | 4321        | 4321       |

The hash key of the node that should receive the packet is:

```Text
hash_key = (3405803777 ^ 3405803778 ^ (1234 << 16) ^ 1234 ^ (4321 << 8) ^ 4321) % 21
         = (3405803777 ^ 3405803778 ^ 80871424 ^ 1234 ^ 1106176 ^ 4321) % 21
         = 20
```

Therefore, the packet gets forwarded to server `0` on the first hop, if the packet is not a [**`TCP SYN`**](https://datatracker.ietf.org/doc/html/rfc9293#section-3.1-6.14.2.14.1) packet and there isn't and established connection for it, then it gets redirected to server `6`.
