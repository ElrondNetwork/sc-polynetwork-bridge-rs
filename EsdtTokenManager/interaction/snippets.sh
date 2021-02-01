ALICE="/home/elrond/elrond-sdk/erdpy/testnet/wallets/users/alice.pem"
ADDRESS=$(erdpy data load --key=address-testnet-esdtTokenManager)
DEPLOY_TRANSACTION=$(erdpy data load --key=deployTransaction-testnet-esdtTokenManager)
PROXY=http://localhost:7950 # For public testnet, replace with https://testnet-gateway.elrond.com
CHAIN_ID=local-testnet
PROJECT_HARDCODED="/home/elrond/sc-polynetwork-bridge-rs/EsdtTokenManager"

# CROSS_CHAIN_MANAGEMENT_ADDRESS=$(erdpy data load --key=address-testnet-crossChainManagement)
# CROSS_CHAIN_MANAGEMENT_ADDRESS_DECODED=$(erdpy wallet bech32 --decode ${CROSS_CHAIN_MANAGEMENT_ADDRESS})

# To get tx result, go to http://localhost:7950/transaction/tx_hash_here?withResults=true

deploy() {
    # --arguments 0x${CROSS_CHAIN_MANAGEMENT_ADDRESS_DECODED}
    erdpy --verbose contract deploy --project=${PROJECT_HARDCODED} --nonce=${alice_nonce} --pem=${ALICE} --gas-limit=200000000 --send --outfile="deploy-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(erdpy data parse --file="deploy-testnet.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="deploy-testnet.interaction.json" --expression="data['emitted_tx']['address']")

    erdpy data store --key=address-testnet-esdtTokenManager --value=${ADDRESS}
    erdpy data store --key=deployTransaction-testnet-esdtTokenManager --value=${TRANSACTION}

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

# arguments: amount
issueWrappedEgld() {
    erdpy --verbose contract call ${ADDRESS} --nonce=${alice_nonce} --pem=${ALICE} --gas-limit=100000000 --function="performWrappedEgldIssue" --value=5000000000000000000 --arguments $1 --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

getWrappedEgldTokenIdentifier() {
    erdpy --verbose contract query ${ADDRESS} --function="getWrappedEgldTokenIdentifier" --proxy=${PROXY}
}

# arguments: token identifier
getTokenAmount() {
    erdpy --verbose contract query ${ADDRESS} --function="getTotalWrappedRemaining" --arguments $1 --proxy=${PROXY}
}

# arguments: token identifier
wasTokenIssued() {
    erdpy --verbose contract query ${ADDRESS} --function="wasTokenIssued" --arguments $1 --proxy=${PROXY}
}

# arguments: eGLD payment. 1 eGLD per 1 wrapped eGLD (Note: must pass with 18 zeroes)
wrapEgld() {
    erdpy --verbose contract call ${ADDRESS} --nonce=${alice_nonce} --pem=${ALICE} --gas-limit=1000000000 --function="wrapEgld" --value=$1 --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# ESDT payment
# arguments: token identifier, amount
unwrapEgld() {
    erdpy --verbose contract call ${ADDRESS} --nonce=${alice_nonce} --pem=${ALICE} --gas-limit=1000000000 --function="unwrapEgld" --arguments $1 $2 --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# arguments: token identifier, amount
mintTokens() {
    erdpy --verbose contract call ${ADDRESS} --nonce=${alice_nonce} --pem=${ALICE} --gas-limit=1000000000 --function="mintEsdtToken" --arguments $1 $2 --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# arguments: token identifier, amount
burnTokens() {
    erdpy --verbose contract call ${ADDRESS} --nonce=${alice_nonce} --pem=${ALICE} --gas-limit=1000000000 --function="burnEsdtToken" --arguments $1 $2 --send --proxy=${PROXY} --chain=${CHAIN_ID}
}
