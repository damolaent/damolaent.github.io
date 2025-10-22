---
title: "Build A Deflationary ERC20 Token From Scratch - No dependencies"
date: "2025-08-12"
author: "Icon The Great"
description: "how to write a deflationary erc20 from scratch with no dependencies"
category: "Ethereum"
---

This post takes a deep dive into ```MyDeflationaryToken```, a Solidity contract that implements an ERC20-like token with a built-in deflationary fee system. The idea is straightforward: every transfer charges a fee, which is split into a burn portion, a treasury portion, and a hodlers reward portion. 

The burn portion permanently reduces the supply, creating a deflationary effect over time.

We will go through the contract’s structure, variables, and functions, explaining what each part does and why it is there.

## GETTING STARTED

To get started we need to have foundry installed on our computer. To install foundry run:
```javascript
# Download foundry installer foundryup
curl -L https://foundry.paradigm.xyz | bash
# Install forge, cast, anvil, chisel
foundryup
# Install the latest nightly release
foundryup -i nightly
``` 

After getting foundry installed, now lets start building our project, first we will create a new directory in our code editor:

```javascript
mkdir deflationary-erc20
cd deflationary-erc20
```

Our ```deflationary-erc20``` will be created, then in our directory, lets run:
```javascript
forge init
```

```forge init``` spin up a new foundry project in our directory ```deflationary-erc20```, the next thing will will do is to delete the ```counter.sol``` file in ```src``` and create a new file named ```MyDeflationaryToken.sol```. You can also go ahead and delete ```counter.t.sol``` and ```counter.s.sol``` in ```test``` and ```script``` folders respectively.

All done? Aye, Let's get building!

#### LICENSE AND COMPILER VERSION

```javascript
//SPDX-License-Identifier: MIT

pragma solidity ^0.8.19;
```

First thing will will do is to indicate the license and solidity version of our contract. ```The SPDX license identifier``` declares that the code is under the ```MIT``` license, which is permissive and widely used in open source.

The ```pragma solidity ^0.8.19;``` statement tells the compiler to use Solidity version ***0.8.19*** or higher, but not ***0.9.0***. Solidity ***0.8.x*** includes built-in overflow and underflow checks, which improve safety.

#### CONTRACT DOCUMENTATION

```javascript
/**
 * @title MyDeflationaryToken
 * @author ICON
 * @notice This contract implements a basic ERC20 token with a transfer fee mechanism.
 * It allows for minting, transferring, and burning tokens, with fees distributed to a treasury wallet,
 * a hodlers distribution wallet, and a burn mechanism.
 * The transfer fee is defined in basis points (1/100th of a percent) and can be set during contract deployment.
 * The contract also includes custom error messages for better clarity and gas efficiency.
 * This contract is designed to be simple and efficient, focusing on the core functionalities of an ERC20 token.
 */
```
The docstring at the top of the contract explains its purpose, author, and main features. You can skip this for now as its not necessary but at the same time can be very important - its useful for both developers and auditors to quickly understand the intent. 

***Make sure you edit the ```@author``` to your dev name. Also dont forget to include those ```NatSpec(/** .... */)``` and you can edit the comments to better explain your contract if you want***

#### CUSTOM ERRORS

```javascript 
error MyDeflationaryToken__CantExceedMaxTransferFee();
error MyDeflationaryToken__AllFeesMustSumUpToTransferFee();
error MyDeflationaryToken__CantExceedTransferFee();
error MyDeflationaryToken__CantBeZeroAddress();
error MyDeflationaryToken__NotOwner();
error MyDeflationaryToken__LesserBalance();
error MyDeflationaryToken__NotApprovedForThisAmount();
error MyDeflationaryToken__TransferFailed();
```

Instead of using ```require``` with strings, the contract uses ***custom errors***. This reduces gas costs because errors store data more efficiently than string messages. Each error has a descriptive name, making it clear what condition failed. We will be using this errors later in our contract, don't bother understanding them for now tho i made them more descriptive that you can grab their functions just by reading it.

#### EVENTS

```javascript
event Transfer(address indexed from, address indexed to, uint256 value);
event Approval(address indexed owner, address indexed spender, uint256 value);
```

These is done to match the ERC20 standard events:

- Transfer is emitted whenever tokens move between accounts or are minted/burned.
- Approval is emitted when a wallet approves another address to spend tokens on its behalf.
Events are crucial for off-chain tracking, like block explorers or dapps.

***In the ERC20 token standard, there are some certain functions, events, variables that should be used in the contract to be considered an ERC20 token.***


#### STATE VARIABLES

Let's specify our state variables.

```javascript
uint256 public transferFee;
uint256 public burnPercent;
uint256 public hodlersPercent;
uint256 public treasuryPercent;
address public immutable treasuryWallet;
address public immutable hodlersDistributionWallet;
```

These store the tokenomics configuration:
- ```transferFee``` is the total fee percentage (in basis points).
- The ```burnPercent```, ```hodlersPercent```, and ```treasuryPercent``` split that total fee.
- The treasury and hodlers wallet addresses are ```immutable```, meaning they are set once at deployment and cannot be changed.

```javascript
    uint256 private constant MAX_TRANSFER_FEE = 1_000; // 10% in basis points
    uint256 private constant PRECISION = 10_000; // 10000 basis points = 100%
```

- ```MAX_TRANSFER_FEE``` is the maximum allowed transfer fee percentage (10% in this case).
- ```PRECISION``` represents the basis point scale (10_000 means percentages are in basis points, so 500 means 5%).

You may be asking why we are using **basis points** instead of just using ```10``` to represent ```10%``` and ```100``` to represent ```100%```, the issue is solidity doesn't support float like other programing languages. If we want to for example, use ```0.1%``` as our ```transferFee```, passing ```0.1``` as a parameter wont work so we have to make use of basis points. ```1_000``` represents ```10%```, ```500``` represents ```5%```, ```10``` represents ```0.1%``` and so one, this is widely used in DeFi.


```javascript 
address public immutable owner;
string public constant name = "IconToken";
string public constant symbol = "ICON";
uint8 public constant decimals = 18;
uint256 private _totalSupply;
```

- ```owner``` is the contract owner (set at deployment).
- ```name```, ```symbol```, and ```decimals``` follow ```ERC20``` metadata standards.
- ```_totalSupply``` stores the total number of tokens in circulation.

```javascript 
mapping(address => uint256) public balances;
mapping(address owner => mapping(address spender => uint256 amount)) public approvals;
```

- ```balances``` maps each address to its token balance.
- ```approvals``` (or allowances) track how much a spender is allowed to spend from an owner’s account.

#### MODIFIERS

We are going to be using ```modifier onlyOwner``` for access control, there are some functions in our contract that we will want only the deployer can call, like the ```mint()``` and ```updateFee()```.

```javascript
modifier onlyOwner() {
    if (msg.sender != owner) {
        revert MyDeflationaryToken__NotOwner();
    }
    _;
}
```

This ensures that certain functions can only be called by the contract owner.

#### CONSTRUCTOR

And then we have a giant constructor, this is for the contract deployer.

```javascript 
    constructor(
        address _treasuryWallet,
        address _hodlersDistributionWallet,
        uint256 _transferFee,
        uint256 _burnPercent,
        uint256 _treasuryPercent,
        uint256 _hodlersPercent
    ) {
        owner = msg.sender;
        treasuryWallet = _treasuryWallet;
        if (_transferFee > MAX_TRANSFER_FEE) {
            revert MyDeflationaryToken__CantExceedMaxTransferFee();
        }
        transferFee = _transferFee;
        burnPercent = _burnPercent;
        treasuryPercent = _treasuryPercent;
        hodlersPercent = _hodlersPercent;
        uint256 allFees = burnPercent + treasuryPercent + hodlersPercent;
        if (allFees != _transferFee) {
            revert MyDeflationaryToken__AllFeesMustSumUpToTransferFee();
        }
        hodlersDistributionWallet = _hodlersDistributionWallet;
    }

```

The ```constructor```:
- Sets owner to the deployer’s address.
- Stores the ```treasury``` and ```hodlers``` wallet addresses.
- Checks that ```_transferFee``` does not exceed ```MAX_TRANSFER_FEE```.
- Stores ```transferFee```, ```burnPercent```, ```treasuryPercent```, and ```hodlersPercent```.
- Calculates the sum of all fee percentages and ensures it equals ```transferFee```.
- Stores the ```hodlersDistributionWallet```.


If any of these conditions fail, the constructor reverts using the relevant custom error.

#### MINT FUNCTION 

Next is the mint function, this allows only the owner i.e the deployer of the contract can call.

```javascript
    function mint(address to, uint256 amount) public onlyOwner {
        if (to == address(0)) {
            revert MyDeflationaryToken__CantBeZeroAddress();
        }
        balances[to] += amount;
        _totalSupply += amount;
        emit Transfer(address(0), to, amount);
    }
```

- Anytime the ```deployer``` calls the ```mint()``` function, they will pass in an ```amount``` they want to mint and the address ```to``` that they want to mint too, Remember ```onlyOwner``` can call this function this is why it's restricted to the owner using the ```onlyOwner``` passed in the function right after public visibility.

- Checks that to is not the zero address, this is crucial, we don't want to mint token to ```address(0)``` better known as burn address.
- We use ```balances[to] += amount;``` to increases the recipient’s balance in our balance mapping we created ealier and then add the amount minted to ```_totalSupply``` balance too.
- Emits a Transfer event from the zero address to indicate minting.

#### TRANSFER FUNCTION 

This is the function that allows transfers of certain amount of our token from one address to the other.
```javascript
function transfer(address receiver, uint256 amount) public returns (bool) {
        if (balances[msg.sender] < amount) {
            revert MyDeflationaryToken__LesserBalance();
        }
        if (receiver == address(0)) {
            revert MyDeflationaryToken__CantBeZeroAddress();
        }
        uint256 fee = (amount * transferFee) / PRECISION;
        uint256 burnShare;
        uint256 treasuryShare;
        uint256 hodlersShare;
        if (fee > 0 && transferFee > 0) {
            burnShare = (fee * burnPercent) / transferFee;
            treasuryShare = (fee * treasuryPercent) / transferFee;
            hodlersShare = fee - burnShare - treasuryShare; // remainder to hodlers
        } else {
            burnShare = 0;
            treasuryShare = 0;
            hodlersShare = 0;
        }

        uint256 netAmount = amount - fee;
        balances[receiver] += netAmount;
        balances[treasuryWallet] += treasuryShare;
        balances[hodlersDistributionWallet] += hodlersShare;
        balances[msg.sender] -= amount;
        _totalSupply -= burnShare; // Reduce total supply by the burned amount
        emit Transfer(msg.sender, receiver, netAmount);
        if (treasuryShare > 0) emit Transfer(msg.sender, treasuryWallet, treasuryShare);
        if (hodlersShare > 0) emit Transfer(msg.sender, hodlersDistributionWallet, hodlersShare);
        if (burnShare > 0) emit Transfer(msg.sender, address(0), burnShare);
        return true;
    }

```

This ```transfer``` function sends tokens from the ```sender``` (the person calling the function) to another address, but it also applies a transfer fee that gets split into three parts:

- Burn (tokens destroyed forever)

- Treasury wallet (for the project’s funds)

- Hodlers wallet (distributed to token holders)

##### Step-by-Step Explanation:

###### Function signature
```solidity

function transfer(address receiver, uint256 amount) public returns (bool)
```
- ```receiver```: the person you want to send tokens to.
- ```amount```: how many tokens you want to send.
- ```returns (bool)```: returns ```true``` if the transfer is successful.

###### 1. Check the sender’s balance
```solidity

if (balances[msg.sender] < amount) {
    revert MyDeflationaryToken__LesserBalance();
}
```
If the ```sender``` don’t have enough tokens, the transaction fails with a custom error ```MyDeflationaryToken__LesserBalance```.

###### 2. Prevent sending to the zero address
``` solidity

if (receiver == address(0)) {
    revert MyDeflationaryToken__CantBeZeroAddress();
}
```
The zero address ```(0x000...000)``` is like a black hole for tokens. This check prevents accidental loss.

###### 3. Calculate the fee
```solidity

uint256 fee = (amount * transferFee) / PRECISION;
transferFee is a percentage (like 200 for 2% if PRECISION is 10,000).
```

This line calculates the fee to deduct from the ```transfer```.

###### 4. Split the fee into parts
```solidity

if (fee > 0 && transferFee > 0) {
    burnShare = (fee * burnPercent) / transferFee;
    treasuryShare = (fee * treasuryPercent) / transferFee;
    hodlersShare = fee - burnShare - treasuryShare;
} else {
    burnShare = 0;
    treasuryShare = 0;
    hodlersShare = 0;
}
```
- ```burnShare```: part of the fee that gets destroyed.

- ```treasuryShare```: goes to the project’s treasury.

- ```hodlersShare```: goes to the special wallet for rewarding holders.

If there’s no fee, all shares are set to 0.

###### 5. Calculate the net amount to send
```solidity
uint256 netAmount = amount - fee;
```
This is the actual amount the receiver will get after subtracting the fee.

###### 6. Update balances

```solidity

balances[receiver] += netAmount;
balances[treasuryWallet] += treasuryShare;
balances[hodlersDistributionWallet] += hodlersShare;
balances[msg.sender] -= amount;
_totalSupply -= burnShare;
```
- Add tokens to the receiver’s balance.

- Add the treasury and hodler’s shares to their wallets.

- Subtract the full amount from the sender (because the fee is also taken from them).

Reduce ```_totalSupply``` by the burn amount (permanently removing tokens).

###### 7. Emit Transfer events
```solidity

emit Transfer(msg.sender, receiver, netAmount);
if (treasuryShare > 0) emit Transfer(msg.sender, treasuryWallet, treasuryShare);
if (hodlersShare > 0) emit Transfer(msg.sender, hodlersDistributionWallet, hodlersShare);
if (burnShare > 0) emit Transfer(msg.sender, address(0), burnShare);
```
Transfer events let blockchain explorers (like Etherscan) and frontends track token movements. Even burning is logged as a ```transfer``` to the ```zero``` address.

###### 8. Return success
```solidity

return true;
The function ends successfully and returns true.
```

###### Example:
If Alice sends 100 tokens to Bob with:

- ```transferFee``` = 5%

- ```burnPercent``` = 40%

- ```treasuryPercent``` = 40%

The rest goes to hodlers.

###### Then:

- Fee = 5 tokens.

- Burn = 2 tokens.

- Treasury = 2 tokens.

- Hodlers = 1 token.

Bob gets 95 tokens.

Supply decreases by 2 tokens.

#### TRANSFER FROM FUNCTION
This performs almost the same function as the ```transfer()``` but here someone or another contract can transfer a user tokens on their behalf.
```javascript
function transferFrom(address sender, address receiver, uint256 amount) public returns (bool) {
        if (approvals[sender][msg.sender] < amount) {
            revert MyDeflationaryToken__NotApprovedForThisAmount();
        }
        if (balances[sender] < amount) {
            revert MyDeflationaryToken__LesserBalance();
        }
        if (sender == address(0) || receiver == address(0)) {
            revert MyDeflationaryToken__CantBeZeroAddress();
        }
        uint256 fee = (amount * transferFee) / PRECISION;
        uint256 burnShare;
        uint256 treasuryShare;
        uint256 hodlersShare;

        if (fee > 0 && transferFee > 0) {
            burnShare = (fee * burnPercent) / transferFee;
            treasuryShare = (fee * treasuryPercent) / transferFee;
            hodlersShare = fee - burnShare - treasuryShare; // remainder to hodlers
        } else {
            burnShare = 0;
            treasuryShare = 0;
            hodlersShare = 0;
        }

        uint256 netAmount = amount - fee;
        balances[receiver] += netAmount;
        balances[treasuryWallet] += treasuryShare;

        balances[hodlersDistributionWallet] += hodlersShare;

        balances[sender] -= amount;
        approvals[sender][msg.sender] -= amount; // Decrease the allowance
        _totalSupply -= burnShare; // Reduce total supply by the burned amount
        emit Transfer(sender, receiver, netAmount);
        if (treasuryShare > 0) emit Transfer(sender, treasuryWallet, treasuryShare);
        if (hodlersShare > 0) emit Transfer(sender, hodlersDistributionWallet, hodlersShare);
        if (burnShare > 0) emit Transfer(sender, address(0), burnShare);

        return true;
    }
```

Steps:
- Checks ```allowance``` from sender to ```msg.sender```.
- Checks ```sender’s balance```.
- Ensures neither ```address``` is the ```zero``` address.
- Calculates the same ```fee``` splits as ```transfer()```.
- Updates ```balances``` accordingly.
- Decreases the ```spender’s allowance```.
- Reduces ```_totalSupply``` by the burn amount.
- Emits the same set of Transfer events.

#### APPROVE FUNCTION
Now, Let's make sure people can approve some particular contract to use ```transferFrom()``` on thier tokens.
```javascript
    function approve(address spender, uint256 amount) public returns (bool) {
        approvals[msg.sender][spender] = amount;
        emit Approval(msg.sender, spender, amount);
        return true;
    }
```


###### Function Overview
```solidity
function approve(address spender, uint256 amount) public returns (bool)
```
- ```spender``` → the address you want to give permission to spend your tokens.

- ```amount``` → how many tokens they’re allowed to spend on your behalf.

- ```returns (bool)``` → returns ```true``` if successful.

This is part of the ```ERC-20``` token standard and is used before someone calls ```transferFrom()```.


###### Step-by-Step:

###### 1. Set the allowance
``` solidity
approvals[msg.sender][spender] = amount;
```
- ```approvals``` is a mapping that stores how much each spender is allowed to spend from each owner.

- ```msg.sender``` is the owner (the one granting permission).

- ```spender``` is the authorized address.

This line sets the allowed amount to amount.

###### Example:

If Alice calls:
```solidity
approve(Bob, 50);
```

###### That means:
```solidity
approvals[Alice][Bob] = 50;
```
So ```Bob``` can now move up to ```50``` tokens from ```Alice’s``` balance using ```transferFrom()```.

###### 2. Emit an ```Approval``` event
```solidity
emit Approval(msg.sender, spender, amount);
```
This logs the approval on the blockchain. Wallets and dApps (like Uniswap) watch for this event so they know when they have permission.

###### 3. Return ```success```
```solidity
return true;
```
Returns ```true``` to confirm the approval worked.

Remember this function doesn't transfer tokens — it only sets permission. Once approved, the spender can call ```transferFrom()``` until they use up the allowance, or the owner changes it with another ```approve()``` call.

If you ```approve``` again, it overwrites the previous ```allowance```.

###### Example in Action:
- Alice has 100 tokens.
- Alice calls:
```solidity
approve(Bob, 40);
```
→ Now Bob is allowed to take up to 40 tokens from Alice.

###### Bob can now call:

```solidity
transferFrom(Alice, Charlie, 25);
```
→ Charlie gets 25 tokens, Bob’s remaining allowance = 15.

#### UPDATE FEES function

The purpose of the ```updateFee()``` is to let the contract owner change the transfer fee and how that fee is split between burn, treasury, and hodlers.

```javascript
    function updateFees(
        uint256 _newTransferFee,
        uint256 _newBurnPercent,
        uint256 _newTreasuryPercent,
        uint256 _newHodlersPercent
    ) public onlyOwner {
        if (_newTransferFee > MAX_TRANSFER_FEE) {
            revert MyDeflationaryToken__CantExceedMaxTransferFee();
        }
        transferFee = _newTransferFee;
        burnPercent = _newBurnPercent;
        treasuryPercent = _newTreasuryPercent;
        hodlersPercent = _newHodlersPercent;
        uint256 allFees = burnPercent + treasuryPercent + hodlersPercent;
        if (allFees != _newTransferFee) {
            revert MyDeflationaryToken__AllFeesMustSumUpToTransferFee();
        }
    }
```

###### Step-by-step

- Access control

Uses ```onlyOwner``` modifier → only deployer/owner can call.

- Max fee check

If ```_newTransferFee > MAX_TRANSFER_FEE (10%)```, it reverts.

- Update state

Sets new ```transferFee```, ```burnPercent```, ```treasuryPercent```, ```hodlersPercent```.

- Ensures the same checkings and validations as the previous ```constructor()``` when deploying. using this function affects all future transfers, but not past ones.

#### INCREASE AND DECREASE ALLOWANCE FUNCTION
```javascript
    function increaseAllowance(address spender, uint256 addedValue) public onlyOwner returns (bool) {
        approvals[msg.sender][spender] += addedValue;
        emit Approval(msg.sender, spender, approvals[msg.sender][spender]);
        return true;
    }

    function decreaseAllowance(address spender, uint256 subtractedValue) public onlyOwner returns (bool) {
        if (approvals[msg.sender][spender] < subtractedValue) {
            revert MyDeflationaryToken__NotApprovedForThisAmount();
        }
        approvals[msg.sender][spender] -= subtractedValue;
        emit Approval(msg.sender, spender, approvals[msg.sender][spender]);
        return true;
    }
```

These two function does so simple and almost similar thing, ```increaseAllowance()``` to obviously increase the spender ```allowance```. ```addedValue``` is added to existing ```approvals[msg.sender][spender]```, emits ```Approval``` with the new total allowance and return ```true``` for success.

The ```decreaseAllowance()``` on the other hand to decrease the spender allwonces. It gets the current allowance, If trying to subtract more than allowed, ```revert```. If not, subtract ```subtractedValue``` from allowance. Emit ```Approval``` with the new allowance.

Return true.

#### VIEW FUNCTION
- ```totalSupply()``` returns the current total supply.
- ```balanceOf(address user)``` returns a specific wallet’s token balance.
- ```allowance(address _owner, address spender)``` returns the current approved amount for a spender.



And Voila! We’ve just built a deflationary ERC-20 token with a transfer fee mechanism that burns tokens, funds the treasury, and rewards holders.

This mechanism can help create scarcity while funding development and incentivizing long-term holding. It’s a great fit for projects that want sustainable tokenomics.

You can try deploying this contract on a testnet, tweak the fee percentages, or extend it with staking features. If you build something with it, share your results. I’d love to see them!

Also remember we use no battles tested dependencies like Openzeppelin here. I'm not perfect, if you spot any bug in this contract, feel free to PR on my Github (Link below)

#### THANKS FOR READING!!

Check the full code here on [Github](https://github.com/IconTheGreat/deflationary-erc20)

And don't forget to follow me on my socials to keep up on what im building next [Twitter](https://x.com/Icon_The_Great)

ciao ciao!!




