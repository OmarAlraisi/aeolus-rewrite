# <span style="font-size: 28px"><span style="text-transform: uppercase;"><span style="font-size: 36px">a</span>eolus</span> - Forward Tables & Hashing</span>

Aeolus uses a simple two-array structure for the [`forwarding table`](https://www.baeldung.com/cs/routing-vs-forwarding-tables#:~:text=A%20forwarding%20table%20simply%20forwards,%2C%20and%20host%2Dspecific%20methods.), and uses a hash of the [`4-tuple`](https://www.cse.iitb.ac.in/~cs348m/notes/lec08.txt#:~:text=TCP%20uses%204%2Dtuple%20(source%20IP%2C%20source%0A%20%20port%2C%20destination%20IP%2C%20destination%20port)) to index the table.

## Forward Table

TODO

Assuming we have `n` servers, each hop table of the forwarding table should have `(n * (n - 1))` entries.

For example, 5 servers (servers 0 - 4) and currently we are draining server `0`:
```
{
   "first":  [1, 2, 3, 4, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4],
   "second": [0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4]
}
```

## Hashing

TODO

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
