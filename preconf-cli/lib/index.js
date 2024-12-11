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
    to: "0xe21B68796fF5BdbeE8f7285B2222AAd3c6c607b5",
    value: ethers.parseEther("0.1"),
    maxFeePerGas: ethers.parseUnits("200", "gwei"),
    maxPriorityFeePerGas: ethers.parseUnits("30", "gwei"),
  };
  const estimatedGas = await wallet.estimateGas(tx);
  tx.gasLimit = estimatedGas;

  const populated = await wallet.populateCall(tx);
  const signedTx = await wallet.signTransaction(populated);
  console.log('signedtx', signedTx)
  
  const txHash = keccak256(signedTx);
  const slot = proposer.slot
  // const slot = 1
  const slotBytes = numberToLittleEndianBytes(slot);
  const txHashBytes = hexToBytes(txHash);

  const message = new Uint8Array(slotBytes.length + txHashBytes.length);
  message.set(slotBytes);
  message.set(txHashBytes, slotBytes.length);
  const messageDigest = keccak256(message);
  const signature = wallet.signingKey.sign(messageDigest).serialized;

  const sidecar_message = new Uint8Array(slotBytes.length + txHashBytes.length);
  // const holeskyGenesisTime = 1695902400
  // const decayStartTimestamp = (holeskyGenesisTime + Number(slot)*12) * 1000

  if(proposer.source === 'interstate'){
 
    await axios.post(
      `${process.env.ROUTER_URL}/submit`,
      {
        proposer,
        signed_tx:signedTx,
        sidecar_signature:signature,
        signature,
        signer
      }
    )
    console.log(`preconfirmation will be sent for slot ${slot} to validator with index 10 at ${proposer.source}`);

  }else if(proposer.source === 'bolt'){
    sidecar_message.set(txHashBytes);
    sidecar_message.set(slotBytes, txHashBytes.length);
    const messageDigest = keccak256(sidecar_message);
    const sidecar_signature = wallet.signingKey.sign(messageDigest).serialized;

    const signer = await wallet.getAddress()
    const body = {
      proposer,
      signed_tx:signedTx,
      sidecar_signature,
      signature,
      signer
    }
    console.log('boday', body)
    try{
      await axios.post(`${process.env.ROUTER_URL}/submit`, body, {headers: {'Content-Type': 'application/json'}});
    }catch(err){

    }
 
    console.log(`preconfirmation with tx: ${txHash} will be sent for slot ${slot} to validator with index ${proposer.validator_index} at ${proposer.source}`);
  
  }else if(proposer.source === 'premev'){
    try{
      const signer = await wallet.getAddress()
      await axios.post(
        `${process.env.ROUTER_URL}/submit`,
        {
          proposer,
          signed_tx:signedTx,
          sidecar_signature:signature,
          signature,
          signer
        }
      )
     console.log(`preconfirmation with tx: ${txHash} will be sent for slot ${slot} to validator with index ${proposer.validator_index} at ${proposer.source}`);

    }catch(err){
      console.log('err mev-premev', err)
    }
  }
}

export async function sendPreconfirmationToInterstateSidecar(
  wallet,
  nonce,
  chainId,
){ 
  const { data } = await axios.get(`${process.env.DEVNET_BEACON_RPC}/eth/v1/beacon/headers`);
  const slot = Number(data.data[0].header.message.slot) + 4;
  const slot1 = Number(data.data[0].header.message.slot) + 8;

  const sender = await wallet.getAddress();

  let nextNonce = nonce

  // Define the transaction
  const tx = {
    chainId: chainId,
    from: sender,
    to: "0xdeaDDeADDEaDdeaDdEAddEADDEAdDeadDEADDEaD",
    value: ethers.parseEther("0.0048560"),
    maxFeePerGas: ethers.parseUnits("300", "gwei"),
    maxPriorityFeePerGas: ethers.parseUnits("40", "gwei"),
    data: "0xdeadbeef",
  };
  
  // Define the transaction
  const tx1 = {
    chainId: chainId,
    from: sender,
    to: "0x8aC112a5540f441cC9beBcC647041A6E0D595B94",
    value: ethers.parseEther("0.0048560"),
    maxFeePerGas: ethers.parseUnits("300", "gwei"),
    maxPriorityFeePerGas: ethers.parseUnits("40", "gwei"),
    data: "0xbeefdead",
  };
  await sendPreconfTx(tx, nextNonce, slot, wallet, process.env.SIDECAR_URL1);
  // await sendPreconfTx(tx1, ++nextNonce, slot, wallet, process.env.SIDECAR_URL2);

  // await sendPreconfTx(tx, ++nextNonce, slot1, wallet, process.env.SIDECAR_URL1);
  // await sendPreconfTx(tx1, ++nextNonce, slot1, wallet, process.env.SIDECAR_URL2);
}


export async function sendPreconfirmationToInterstateGateway(
  wallet,
  nonce,
  chainId,
){ 
  const { data } = await axios.get(`${process.env.DEVNET_BEACON_RPC}/eth/v1/beacon/headers`);
  const slot = Number(data.data[0].header.message.slot) + 10;
  const sender = await wallet.getAddress();

  // Define the transaction
  const tx = {
    nonce,
    chainId: chainId,
    from: sender,
    to: "0xdeaDDeADDEaDdeaDdEAddEADDEAdDeadDEADDEaD",
    value: ethers.parseEther("0.0048560"),
    maxFeePerGas: ethers.parseUnits("300", "gwei"),
    maxPriorityFeePerGas: ethers.parseUnits("40", "gwei"),
    data: "0xdeadbeef",
  };

  const estimatedGas = await wallet.estimateGas(tx);
  tx.gasLimit = estimatedGas;

  const populated = await wallet.populateCall(tx);
  const signedTx = await wallet.signTransaction(populated);

  const txHash = keccak256(signedTx);

  const {data:dataRes} = await axios.post(
    `${process.env.INTERSTATE_GATEWAY}/api/v1/send_preconf`,
    {
      tx:signedTx,
      sender
    }
  );

  console.log('data' , dataRes)

  console.log(`sent preconfirmation tx: ${txHash} at slot ${dataRes.slot} at nonce ${nonce}`)
}

async function sendPreconfTx(tx, nonce, slot, wallet, url) {
  console.log('nonce', nonce, url);
  tx.nonce = nonce
  const estimatedGas = await wallet.estimateGas(tx);
  tx.gasLimit = estimatedGas;

  const populated = await wallet.populateCall(tx);
  const signedTx = await wallet.signTransaction(populated);

  const txHash = keccak256(signedTx);

  const sender = await wallet.getAddress();
  console.log('sending')
  await axios.post(
    `${url}/api/v1/preconfirmation`,
    {
      tx:signedTx,
      sender,
      slot
    }
  );

  console.log(`sent preconfirmation tx: ${txHash} at slot ${slot} at nonce ${nonce}`)
}

export async function getProposer(){
  try{
    const { data } = await axios.get(`${process.env.ROUTER_URL}/proposer`)
    if(data) return data
    else return
  }catch(err){
    console.log('err', err)
    return
  }
}

export async function getWallet(network, pvk){
  if(network==="mainnet"){
    const provider = new ethers.JsonRpcProvider(process.env.MAINNET_RPC);
    const wallet = new ethers.Wallet(pvk, provider);
    return wallet;
  }else if(network==="holesky"){
    const provider = new ethers.JsonRpcProvider(process.env.HOLESKY_RPC);
    const wallet = new ethers.Wallet(pvk, provider);
    return wallet;
  }else if(network==="devnet"){
    const network = new Network('kurtosis', 3151908)
    const provider = new ethers.JsonRpcProvider(process.env.DEVNET_RPC, network, { staticNetwork: network });
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

// BigInt.prototype.toJSON = function () {
//   const int = Number.parseInt(this.toString());
//   return int ?? this.toString();
// };