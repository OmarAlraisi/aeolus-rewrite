# <span style="font-size: 28px"><span style="text-transform: uppercase;"><span style="font-size: 36px">a</span>eolus</span> - Forward Tables & Hashing</span>

Aeolus uses a simple two-array structure for the [`forwarding table`](https://www.baeldung.com/cs/routing-vs-forwarding-tables#:~:text=A%20forwarding%20table%20simply%20forwards,%2C%20and%20host%2Dspecific%20methods.), and uses a hash of the [`4-tuple`](https://www.cse.iitb.ac.in/~cs348m/notes/lec08.txt#:~:text=TCP%20uses%204%2Dtuple%20(source%20IP%2C%20source%0A%20%20port%2C%20destination%20IP%2C%20destination%20port)) to index the table.

## Forward Table

TODO

When a new packet arrives, we generate a [`hash`](#hashing) key which is used to retrieve the destination server from the forwarding table.

The forwarding table consists of two arrays, representing the first and second hop destinations. If we have `s` servers (`0 to s-1`), each array will consist of `s * floor(s / 2)` entries (<i>`num_entries`</i>), where each server will appear `floor(s / 2)` (<i>`freq`</i>) times in the array.

When draining a server `x`, it becomes an <i>**always-second hop**</i> server. To remove the server `x` from the first hop array we perform the following:

```Text
    - Find the first position of server 'x'. (idx_x)
    - Calculate the index's pre-value. (idx) - [idx = idx_x - freq]
    - Loop freq times and perform:
        idx = (idx + (2 * freq)) % num_entries
        first[idx_x] = first[idx]
        idx_x = idx_x + 1
```

And once `x` is draind, we replace every occurrence of `x` in the second hop array with the server at the same index but in the first hop array. - Assume `x` occurred at index `i`, then `second[i] = first[i]`.

---

#### Example:

Assume we have `7` servers, the forwarding table arrays will have `7 * floor(7 / 2) = 21` entries. When all servers are healthy, the forwarding table will look as such:

```Text
first  = [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 6]
second = [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 6]
```

Now, let's say we want to drain server `4`.

```Text
idx_x = 12
idx = 12 - 3 = 9
loop 3 times:
    1:
        idx = (9 + (2 * 3)) % 21 = 15
        first[12] = first[15] = 5
        idx_x = 12 + 1 = 13

    2:
        idx = (15 + (2 * 3)) % 21 = 0
        first[13] = first[0] = 0
        idx_x = 13 + 1 = 14

    3:
        idx = (0 + (2 * 3)) % 21 = 6
        first[14] = first[6] = 2
        idx_x = 14 + 1 = 15
```

The resulting forwarding table will look like:

```Text
first  = [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 5, 0, 2, 5, 5, 5, 6, 6, 6]
second = [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 6]
```

And once server `4` is drained, we can replace all its occurrences in second array with its first hop destination:

```Text
first  = [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 5, 0, 2, 5, 5, 5, 6, 6, 6]
second = [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 5, 0, 2, 5, 5, 5, 6, 6, 6]
```

**This doesn't work.** How would you drain servers `5`, `0`, or `2` now?

## Hashing

TODO

`hash_key = (src_ip | dest_ip | (src_port as u32) | (dest_port as u32)) % (n * (n - 1))`

Assume we have a packet with the following info:

| Key       | Value       |
| --------- | ----------- |
| src_ip    2,| 203.0.113.1 |
| src_port  | 1234        |
| dest_ip   | 203.0.113.2 |
| dest_port | 4321        |
```
hash_key = (3405803777 | 3405803778 | (1234 as u32) | (4321 as u32)) % (5 * (5 - 1))
hash_key = (3405805043) % 20
hash_key = 3
```

Therefore this packet gets forwarded to server `4` for the first hop, if no established connection, then gets forwarded to server `0`.
