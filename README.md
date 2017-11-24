# routing-actor

A actor based simulator for a new type of routing system that (hopefully) provides:
- Constant routing table state
- Fast forwarding (no memory lookups in the forwarding plane)
- Unlimited, globally addressable address allocation
- Efficient routing

See [here](https://github.com/kingoflolz/routing-actor/wiki) to see how it is achieved

Status:
- [x] Scaffolding for ergonomic actor based network simulation
- [x] DHT system to look up NC
- [ ] NC system
- [ ] Contextual bandit

Future (likely not in this repo):
- [ ] Private routes
- [ ] Reputation based metric advertisment
- [ ] Blockchain based billing
