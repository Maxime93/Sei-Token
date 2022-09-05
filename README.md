# Sei exercise notes for reviewers

The below is a walkthrough of my thinking, resources used and bumps I hit along the road. Learning Rust was fun. The CosmWasm framework for smart contract development is neat as well. This exercise was a good intro to this world.

My lack of knowledge on the Rust language is what limited me the most. Did a lot of bootstrapping, and had to keep things simple to keep the compiler happy. I'm afraid you won't see any neat Rust syntax here, and there are probably things I have not written "the Rust way".

Feel free to reach out with any questions!

## Quick intro about me

As a quick intro about myself.
- Product manager out of college, turned Software/Data engineer (writing data tools for ETL in python), turned Sorftware/Infra (Writing Search backend code in Go).
- Have some EVM smart contract experience through hackathons (Solidity)
- No knowledge of Rust

# Exercise

Coding project specs as communicated to me:
“””
We want you to build a simple Sei smart contract that allows 1-to-2 transfer of the usei token. 

Please create a Sei smart contract (using Cosmwasm 1.0) with the following requirements:
- you should be able to instantiate the contract and set the owner
- you should support a read query to get the owner of the smart contract
- you should support an execute message where an account can send coins to the contract and specify two accounts that can withdraw the coins (for simplicity, split coins evenly across the two destination accounts)
- you should store the withdrawable coins for every account who has non-zero coins in the contract
- you should support an execute message where an account can withdraw some or all of its withdrawable coins
- you should support a read query to get the withdrawable coins of any specified account
- you should write unit tests for all of these scenarios (we should be able to run cargo test and all of the unit tests should pass)

Bonus
- Implement a fee structure for the transfer contract, where each send incurs fees that are collectable by the contract owner
“””
## Getting falmiliar with Rust

Started reading the docs: https://docs.cosmwasm.com/docs/1.0/
Then stumbled upon this tutorial creating a new contract from scratch: https://www.youtube.com/watch?v=VTjiC4wcd7k. Decided to follow that tutorial for project setup and adjust code as needed to fit the coding project specs. This video is recent (Jun 22, 2021) from the official Cosmos Youtube channel, and presented by someone who seems to be involved in cosmwasm development. This makes me confident I will get good information out of this.

I also stumbled upon this repo: https://github.com/InterWasm/cw-contracts/tree/main/contracts which has some good examples I might be able to re-use.
At that point I started setting up the environment.

## Setting up environement

Followed the guide below, everything went well.
https://docs.cosmwasm.com/docs/1.0/getting-started/intro/

Installed cargo generate `cargo install cargo-generate` and set up a new project:
`cargo generate --git https://github.com/CosmWasm/cosmwasm-template.git --name sei-token`

When that was done and I had a blank slate, added and committed the code, and pushed it to this repo.

## "Instantiate the contract and set the owner support a read query to get the owner of the smart contract"

See PR: **https://github.com/Maxime93/sei-token/pull/1**

Starting by cleaning all the messages to have a blank slate in `src/msg.rs`. Cleaning `src/contract.rs` as well with “unimplemented” keyword in the `instantiate`, `execute` and `query` functions. Then discarding existing tests, and deleting some extra functions that were there which are said to be “boilerplate”.

At this point from the little research I was able to make I understand big picture:
- `contract.rs` is where we have the business logic
- `state.rs` is where we define the data objects that will live on chain
- `msg.rs` is where we define how we will interact with the contract (messages users can sed / expect to receive from the contract)

Therefore in `state.rs` I set a new public constant to store the owner information `pub const OWNER: Item<Addr> = Item::new("owner”);`

In `msg.rs` I added a new message for instantiation:
```
pub struct InstantiateMsg {
    pub owner: String,
}
```

In `contract.rs` I wrote the business logic that sets the owner upon instantiation of the contract:
```
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let owner = deps.api.addr_validate(&msg.owner)?;
    OWNER.save(deps.storage, &owner)?;
    let res = Response::new();
    Ok(res)
}
```
(One could create a contract and set another address as the owner).

Also I’ve updated the `query` function so it can return the owner:
```
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env) -> StdResult<Binary> {
    let owner = OWNER.load(deps.storage)?;
    to_binary(&owner)
}
```

Now as I am looking at implementation for the rest of the coding challenge, I realize I could do thing a little better in my previous PR. Right now I a set up to support only one query, and I want to support more.
Also looking at some contract examples, I realize I am not storing contract owner information the same way other templates are doing it. I worry this might block me going forward so I am making those changes now.

See PR: **https://github.com/Maxime93/sei-token/pull/2**

I came across some of the contracts in https://github.com/InterWasm/cw-contracts/tree/main/contracts and realized they pretty much all had a `Config` data object in `state.rs` that contained owner information. These contracts also have some neat logic that sets the creator of the contract as owner, if no owner is provided in the instantiate message.

I added a `match` expression in the `query` function of `contract.rs` which will make adding new queries easier.

Also updated the tests which is now basically a copy/paste of what I found in the example contracts mentioned above. I also added a couple tests that ensure my query function works correctly.

## "Support an execute message where an account can send coins to the contract and specify two accounts that can withdraw the coins (for simplicity, split coins evenly across the two destination accounts)"
## "Store the withdrawable coins for every account who has non-zero coins in the contract"

I am now re-reading the first line of the challenge:
“We want you to build a simple Sei smart contract that allows 1-to-2 transfer of the usei token. “

I realize only now that I am going to need the equivalent of the ERC20 contract in CosmWasm. After a bit of research I found this example of the official github: https://github.com/InterWasm/cw-contracts/tree/main/contracts/cw20-pot
That should help with the rest of the challenge.

I see that `contract.rs` imports `use cw20::{Cw20Contract, Cw20ExecuteMsg, Cw20ReceiveMsg};`
So I’m going to need that cw20 package in my project as well.

I add `cw20 = "0.13.2”` in `Cargo.tolm` under dependencies and run `cargo build`. I see `Compiling cw20 v0.13.4` message, so I assume I can now make use of that contract.

I am going to create a new message `CreatePot` message that will allow users to send money to the contract along with two addresses.

We will be checking:
- If the token is correct (usei)
- If the amount is not null
- If the two addresses are different

See this PR: **https://github.com/Maxime93/sei-token/pull/3**

In this implementation a user won’t be able to add more funds to an existing Pot. Pot will have to be fully emptied (using withdraw below) so a new one can be created.

The Pot data object in `state.rs` only has an amount and an address.
The CreatePot message has two addresses and an amount. So when we receive the message, we split the amount in half, and create two Pots (one for each address). Each Pot will have half the amount sent to the contract.

We then store the Pots created in a map, where the key is an address (formatted as a string - which could potentially lead to prodlems in production? Did not have the time to research), and the value is a Pot.

That way it is easy to look up a Pot, all users need is an address!

## "You should support an execute message where an account can withdraw some or all of its withdrawable coins"

PR: **https://github.com/Maxime93/sei-token/pull/4**

In this PR we add a very simple message:
```
WithdrawPot {
         // The amount you want to withdraw
         amount: Uint128,
}
```

The idea is any address can ask to withdraw the funds locked in their Pot.

The `contract.rs` file implements a new `execute_withdraw_pot` function that checks:
- if the address even has a pot to withdraw from
- if the amount the address is looking to withdraw if <= to the amount in the Pot.

If the withdrawal operation has succeeded, I remove the Pot from POTS map.
In the case that the amount withdrew < amount in the pot, I re-create a new Pot with the adjusted balance.

Some issues with this code that I don’t have time to address because of my lack knowledge of Rust and CosmWasm:
- I’m doing the ledger update after funds might already be off the contract.
- Not taking gas into consideration

## "You should support a read query to get the withdrawable coins of any specified account"

This is already supported. With `query_pot` you can get pot info for any address.

Note that you can see amount available for any address. However you can only withdraw by calling the contract from your address. Withdraw function only takes an amount, we look up the POT by looking at `info.sender`.

## "You should write unit tests for all of these scenarios (we should be able to run cargo test and all of the unit tests should pass)"

This was done all along.



**Below is the standard doc from template.**
# CosmWasm Starter Pack

This is a template to build smart contracts in Rust to run inside a
[Cosmos SDK](https://github.com/cosmos/cosmos-sdk) module on all chains that enable it.
To understand the framework better, please read the overview in the
[cosmwasm repo](https://github.com/CosmWasm/cosmwasm/blob/master/README.md),
and dig into the [cosmwasm docs](https://www.cosmwasm.com).
This assumes you understand the theory and just want to get coding.

## Creating a new repo from template

Assuming you have a recent version of rust and cargo (v1.58.1+) installed
(via [rustup](https://rustup.rs/)),
then the following should get you a new repo to start a contract:

Install [cargo-generate](https://github.com/ashleygwilliams/cargo-generate) and cargo-run-script.
Unless you did that before, run this line now:

```sh
cargo install cargo-generate --features vendored-openssl
cargo install cargo-run-script
```

Now, use it to create your new contract.
Go to the folder in which you want to place it and run:


**Latest: 1.0.0**

```sh
cargo generate --git https://github.com/CosmWasm/cw-template.git --name PROJECT_NAME
````

For cloning minimal code repo:

```sh
cargo generate --git https://github.com/CosmWasm/cw-template.git --branch 1.0-minimal --name PROJECT_NAME
```

**Older Version**

Pass version as branch flag:

```sh
cargo generate --git https://github.com/CosmWasm/cw-template.git --branch <version> --name PROJECT_NAME
````

Example:

```sh
cargo generate --git https://github.com/CosmWasm/cw-template.git --branch 0.16 --name PROJECT_NAME
```

You will now have a new folder called `PROJECT_NAME` (I hope you changed that to something else)
containing a simple working contract and build system that you can customize.

## Create a Repo

After generating, you have a initialized local git repo, but no commits, and no remote.
Go to a server (eg. github) and create a new upstream repo (called `YOUR-GIT-URL` below).
Then run the following:

```sh
# this is needed to create a valid Cargo.lock file (see below)
cargo check
git branch -M main
git add .
git commit -m 'Initial Commit'
git remote add origin YOUR-GIT-URL
git push -u origin main
```

## CI Support

We have template configurations for both [GitHub Actions](.github/workflows/Basic.yml)
and [Circle CI](.circleci/config.yml) in the generated project, so you can
get up and running with CI right away.

One note is that the CI runs all `cargo` commands
with `--locked` to ensure it uses the exact same versions as you have locally. This also means
you must have an up-to-date `Cargo.lock` file, which is not auto-generated.
The first time you set up the project (or after adding any dep), you should ensure the
`Cargo.lock` file is updated, so the CI will test properly. This can be done simply by
running `cargo check` or `cargo unit-test`.

## Using your project

Once you have your custom repo, you should check out [Developing](./Developing.md) to explain
more on how to run tests and develop code. Or go through the
[online tutorial](https://docs.cosmwasm.com/) to get a better feel
of how to develop.

[Publishing](./Publishing.md) contains useful information on how to publish your contract
to the world, once you are ready to deploy it on a running blockchain. And
[Importing](./Importing.md) contains information about pulling in other contracts or crates
that have been published.

Please replace this README file with information about your specific project. You can keep
the `Developing.md` and `Publishing.md` files as useful referenced, but please set some
proper description in the README.
