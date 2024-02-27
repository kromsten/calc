# Astrovault Exchange Wrapper

See `demo.sh` & `demo_implicit.sh` for the latest deployed contract addresses, their stored pair info and examples interactions. T

The implicit version is a bit more developed with routed examples. The existing pools can be created from `pairs_implicit.json`.

Some pair type exist in `pairs.json` but since the logic of the actual functions is identical the example demo file was not recreated as thoroughly

## Types

There are a few characteristics that describes all the pairs.

Pairs can differ by type:
-> Direct Pool is a pair that represent a pool with a matching astrovault contract
-> Routed is a pair that contains information about what pools to use reach quote asset from base asset

Pairs can also be:
-> Unpopulated is the default definition of `Pair` meant to be supplied by admins to be validated & populated
-> Populated is a version with all the necessary info needed for interacting with astrovault contracts

There are also other inner types like `Pool` and `Route` without stripped off the extra information
Pools are almost identical to pairs but routes are a bit trickier:

A rule about assets in routes:

Route -> base and quote assets are not included
PopulatedRoute -> a list of populated pools including base and quote as the first and the last in the list
StoredRoute -> base and quote assets are not included

## Config:

An important new field controlling the behaviour of the app is `allow_implicit` that is set to `false` by default.

If the field is set to `false` only the pairs that had been explicitly created by admins can be used by for querying and swapping.

In case there are routed pairs with hops pools that had not been created explicitly they will be still stored internally for routing purposes. Explicitly creating/overriding a pool at any time will "reveal" it

If the field is set to `true` all those internal pools will be returned as valid pairs alongside explicit ones and any of them can be used for swapping. See examples below

## Messages:

The contract follows exchange interface of calculated.finance for the most part with some minor changes

### Receive

To ensure the same atomic user experience it was necessary to add an additional variant to `ExecuteMsg`.
It now accepts an additional `Receive` variant defined as in Cw20 standard.

That allow the exchange contract be triggered by cw20 contracts upon receiving funds send using `Send` operation.

Without the custom endpoint swapping would happen through 2 operations. Giving allowance to the exchange contract with first transaction and only then triggering swap operation for the contract to use `TransferFrom` internally.

## Pairs Behaviour

### Direct (Pool) Pair

Creating a pair of a direct pool type always creates / overrides one (explicit) pair.
Information about the pool is queried from Astrovault contract to both make sure all specified fields are a valid combinations but also to locally store indices of the assets on Astrovault side

Submitting a new pool with the same base and quote assets query the Astrovault contract and rewrite the storage slot with the new information (or not new)

### Routed Pair

Creating a routed pair creates the pair itself, the pool pairs between each two adjacent hops in the route and also stores routes for the inner denoms of the route. See below to see when they are usable too.

Routes consist of a custom data structure called hops. Hops are for the most side similar to pools except for having notion of main denomination `denom` and "sides" (previous and next `HopInfo`). Each hop must have a `prev` field that tells how to connect to ether base asset of the routed pair or to a previous hop in the route. The last hop must have `next` fields of the same nature that connects it's `denom` to quote asset of the pair.

#### Route examples

Storing one routed pair: A -> (over B - C - D) -> E

Stores:

Explicit routed pair:
A -> E

Implicit pools pairs:
A -> B, B -> C, C -> D, D -> E

Implicit routed pairs:
A -> C, A -> D, B -> D, B -> E, C -> E

Implicit routes are only automatically generated from the route of pair that is being created
They are not constructed using denoms in storage.

E.g. If there is an existing routed pair:

A -> (over B) -> C

Creating a new one:

B -> (over C - D ) -> E

Will generate implicit routes:

B -> D, C -> E

Routes A -> D and A -> E will not

#### Route overriding

Hops (which are technically double sided pools) are populated from the storage first. Supplying a hop with denoms that already stored and pointing to a pool but with custom address and pool type takes no effect. For that purpose the pool must be overridden from prior by creating a direct pair.

In case if some hops of a route are already stored as pools and some are not, the missing ones will be verified and populated by querying astrovault contract (and later saved). Those that did exist from prior wouldn't be updated as described above.

Overriding a route itself by making it longer, shorter or completely changing the assets also overrides implicit routes for denoms in between like described above.

#### Supplying a custom route

Feature is implemented but wasn't mentioned to be optional and hasn't been tested as vigorously as primary features. The binary must be decodable into `Vec` of of hops similar to inner attribute routed pair.

Since after initial creation pools derived from route hops are fetched from storage can be extended to support a simple list of simplified structures.

As a potential improvement `HopInfo` can be turned into Enum with a variant for the current version and one for a simplified one that doesn't require supplying pool type and addresses.

## Storage

Explicitly created pairs have a separate (abstracted) structure for simply storying the type of the pair

Information about pairs are of a pool type is stored as `PopulatedPool` and then converted into appropriate pair

Routed pairs are not technically stored as routed pairs except for a simple flag for explicit pairs. Only the route itself which is a list of `String` denoms between base and quote assets is stored separately. The list is then converted into `PopulatedRoute` and then into appropriate pair

As a general rule of thumb for the helping method

"find*\*" performs implicit/explicit and other checks first and then retrieves
"get*\*"" simply retrieves information if it can

## Other

Registry module might be used for additional check that make sure a provided pool comes from the registry.
At the moment not used anywhere and can be safely deleted alongside the query and stored response of router configuration.
