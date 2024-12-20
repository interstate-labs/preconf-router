import { keccak256, Network } from "ethers";
import { ethers } from "ethers";
import axios from "axios";
import dotenv from 'dotenv';
dotenv.config();

export async function sendPreconfirmation(
  wallet,
  nonce,
  chainId,
  proposer
){
  // Define the transaction
  const tx = {
    chainId: chainId,
    nonce: nonce,
    from: await wallet.getAddress(),
    to: "0xdeaDDeADDEaDdeaDdEAddEADDEAdDeadDEADDEaD",
    value: ethers.parseEther("0.0069420"),
    maxFeePerGas: ethers.parseUnits("200", "gwei"),
    maxPriorityFeePerGas: ethers.parseUnits("30", "gwei"),
    data: "0xdeadbeef",
  };
  const estimatedGas = await wallet.estimateGas(tx);
  tx.gasLimit = estimatedGas;

  const populated = await wallet.populateCall(tx);
  const signedTx = await wallet.signTransaction(populated);
  const txHash = keccak256(signedTx);
  const slot = proposer.slot
  const slotBytes = numberToLittleEndianBytes(slot);
  const txHashBytes = hexToBytes(txHash);
  const message = new Uint8Array(slotBytes.length + txHashBytes.length);
  const holeskyGenesisTime = 1695902400
  const decayStartTimestamp = (holeskyGenesisTime + Number(slot)*12) * 1000




  if(proposer.type === 'interstate'){
      message.set(slotBytes);
      message.set(txHashBytes, slotBytes.length);
    
      const messageDigest = keccak256(message);
      const signature = wallet.signingKey.sign(messageDigest).serialized;
      await fetch(`${proposer.sidecar_url}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          id: "1",
          jsonrpc: "2.0",
          method: "submit_inclusion_preconfirmation",
          messages: [{ slot, tx:signedTx, signature }],
        }),
      }).then((response) => response.json());

      console.log(`preconfirmation will be sent for slot ${slot} to validator with index 10 at url ${proposer.sidecar_url}`);
  }else if(proposer.type === 'bolt'){
    message.set(txHashBytes);
    message.set(slotBytes, txHashBytes.length);
  
    const messageDigest = keccak256(message);
    const signature = wallet.signingKey.sign(messageDigest).serialized;
    const signer = await wallet.getAddress()
    console.log('signer', signer);
    const resp = await fetch(`${proposer.sidecar_url}`, {
      method: "POST",
      headers: { "Content-Type": "application/json", "X-Bolt-Signature":`${signer}:${signature}`},
      body: JSON.stringify({
        id: "1",
        jsonrpc: "2.0",
        method: "bolt_requestInclusion",
        params: [{ slot, txs:[signedTx] }],
      }),
    }).then((response) => response.json());
    console.log('resp', resp);
    console.log(`preconfirmation with tx: ${txHash} will be sent for slot ${slot} to validator with index ${proposer.validator_index} at url ${proposer.sidecar_url}`);
  }else if(proposer.type === 'primev'){
    try{
      const body = {
        "rawTransactions": [signedTx],
        "amount": "100040",
        "blockNumber": slot,
        "decayStartTimestamp": decayStartTimestamp,
        "decayEndTimestamp": decayStartTimestamp + 500,
        "revertingTxHashes": []
      }
      console.log('body', body)
      const {data:resp} = await axios.post(`${process.env.PRIMEV_RPC}/v1/bidder/bid`, body)
      console.log('resp', resp);
    }catch(err){
      console.log('err mev-primev', err)
    }
    
  }
}

export async function getProposer(chainId){
  let slot = await getLatestSlot(chainId);
  let preconfers = []

  if(chainId === 3151908){
    const { data:interstate } = await axios.get(`${process.env.DEVNET_INTERSTATE_GATEWAY_URL}/preconfers`)
    const { data:bolt } = await axios.get(`${process.env.DEVNET_BOLT_GATEWAY_URL}/proposers/lookahead?activeOnly=true&futureOnly=true`)
    
    preconfers = [...interstate.map((v)=>({...v, type:"interstate"})), ...bolt.map((v)=>({...v, type:"bolt"}))]
  }else if(chainId === 17000){
    // const { data:interstate } = await axios.get(`${process.env.HOLESKY_INTERSTATE_GATEWAY_URL}/preconfers`)
   
    const { data:bolt } = await axios.get(`${process.env.HOLESKY_BOLT_GATEWAY_URL}/proposers/lookahead?activeOnly=true&futureOnly=true`)
    preconfers = [...bolt.map((v)=>({...v, type:"bolt"}))]
    
    // const { data: primev } = await axios.get(`${process.env.PRIMEV_RPC}/v1/validator/get_validators`)
    // const primevItems = primev.items
    // let primevProposers = []
    
    // for(let i=0 ; i<Object.keys(primevItems).length ; i++){
    //   const slot = Object.keys(primevItems)[i]
    //   const item = primevItems[slot]
    //   if(item["isOptedIn"]){
    //     primevProposers.push({
    //       slot,
    //       validator_pubkey:item["BLSKey"],
    //       validator_index:undefined,
    //       sidecar_url:undefined,
    //       type:"primev"
    //     })
    //   }     
    // }

    // preconfers.push(...primevProposers)

  }

  if(preconfers.length === 0){
    return null;
  }

  const nextPreconfer = preconfers.find((preconfer)=>preconfer.slot>slot)
  if(!nextPreconfer) return null;
  return nextPreconfer
}

export async function getLatestSlot(chainId){
  const slotResponse = await fetch(`${ chainId===3151908 ? process.env.DEVNET_BEACON_URL : process.env.HOLESKY_BEACON_URL}/eth/v1/beacon/headers/head`).then(
    (response) => response.json(),
  );
  return Number(slotResponse.data.header.message.slot);
}

export async function getWallet(network, pvk){
  if(network==="devnet"){
    const network = new Network('kurtosis', 3151908)
    const provider = new ethers.JsonRpcProvider(process.env.DEVNET_RPC, network, { staticNetwork: network });
    const wallet = new ethers.Wallet(pvk, provider);
    return wallet;
  }else if(network==="holesky"){
    const network = new Network('holesky', 17000)
    const provider = new ethers.JsonRpcProvider(process.env.HOLESKY_RPC, network, { staticNetwork: network });
    const wallet = new ethers.Wallet(pvk, provider);
    return wallet;
  }

}

export function sleep(sec){
  return new Promise(resolve=>{
    setTimeout(()=> resolve(), sec*1000)
  })
}

// Function to convert a number to a little-endian byte array
function numberToLittleEndianBytes(num) {
  const buffer = new ArrayBuffer(8); // Assuming slot_number is a 64-bit integer
  const view = new DataView(buffer);
  view.setUint32(0, num, true); // true for little-endian
  return new Uint8Array(buffer);
}

// Function to decode a hex string to a byte array
function hexToBytes(hex) {
  hex = hex.replace(/^0x/, ""); // Remove "0x" prefix if present
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.substr(i, 2), 16);
  }
  return bytes;
}
