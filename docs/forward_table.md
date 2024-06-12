# <span style="font-size: 28px"><span style="text-transform: uppercase;"><span style="font-size: 36px">a</span>eolus</span> - Forward Tables & Hashing</span>

Aeolus uses a simple two-array structure for the [`forwarding table`](https://www.baeldung.com/cs/routing-vs-forwarding-tables#:~:text=A%20forwarding%20table%20simply%20forwards,%2C%20and%20host%2Dspecific%20methods.), and uses a hash of the [`4-tuple`](https://www.cse.iitb.ac.in/~cs348m/notes/lec08.txt#:~:text=TCP%20uses%204%2Dtuple%20(source%20IP%2C%20source%0A%20%20port%2C%20destination%20IP%2C%20destination%20port)) to index the table.

## Forward Table

When a new packet arrives, we generate a [`hash`](#hashing) key which is used to retrieve the destination server from the forwarding table.

The forwarding table consists of two arrays, representing the first hop and second hop destinations. If we have `s` servers (`0 to s-1`), each array will consist of `s * floor(s / 2)` entries (<i>`num_entries`</i>), where each server will appear `floor(s / 2)` (<i>`freq`</i>) times in the array.

Assuming no server is currently being drained, when draining server `x`, it becomes an <i>**always-second hop**</i> server. To remove the server `x` from the first hop array we perform the following:

- Split the servers into two groups, one with servers who have even indices in an array of all running servers, and the other with for servers with odd indices.

- Iterate over all entries of the first hop array and replace all occurrences of server `x` with the servers in group that doesn't contain `x`.

- Once `x` is drained, remove all occurrences of `x` in the second hop array with the server at the same index in the first hop array.

However, if there is at least one server `y` that is currently being drained, then `x` is ONLY allowed to be drained if it belongs to the same group of `y`.

---

#### Example:

Assume we have `7` servers, the forwarding table arrays will have `7 * floor(7 / 2) = 21` entries. When all servers are running, the forwarding table will look as such:

```Text
first  = [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 6]
second = [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 6]
```

Now, let's say we want to drain server `4`.

```Text
Step I: Split the servers into groups.
    group_a = [0, 2, 4, 6]
    group_b = [1, 3, 5]

Step II: Replace all occurrences of server `4` in the first hop array.
    first  = [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 1, 3, 5, 5, 5, 5, 6, 6, 6]
    second = [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 6]

**Note: At this point, while server `4` is being drained, you can NOT start draining
        servers, `1`, `3`, or `5` since they are the first destination for packets
        that are meant for server `4`. You can only drain servers `0`, `2`, and `6`.

Step III: Remove all occurrences of server `4` from the second hop array.
    first  = [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 1, 3, 5, 5, 5, 5, 6, 6, 6]
    second = [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 1, 3, 5, 5, 5, 5, 6, 6, 6]
```

Now, that there is no server in the `draining` state, you will have to recreate `group_a` and `group_b` once you start draining a new server. They will look like:

```Text
    group_a = [0, 2, 5]
    group_a = [1, 3, 6]
```

TODO: 
- Why this design?
- How to add a new server? `FILLING`

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
