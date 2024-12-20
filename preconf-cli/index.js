#! /usr/bin/env node

// import { program } from 'commander'
// import list from './commands/list'

import { Command } from 'commander';
import { getProposer, getWallet, sendPreconfirmation, getLatestSlot, sleep } from './lib/index.js';

const program = new Command();

async function main() {
  program
    .command('send')
    .description("My Node CLI")
    .requiredOption("-n, --network <type>", "devnet or holesky")
    .requiredOption("-p, --pvk <type>", "Private key to sign a transaction")
    .action(run);
  await program.parseAsync(process.argv);
}


async function run(options) {
  const pvk = options['pvk'];
  const network = options["network"];
  if(!(network==="devnet" || network==="holesky")) {
    console.log('Network can be only "devnet" or "holesky"');
    return;
  }
  const wallet = await getWallet(network, pvk);
  
  // const nonce = 0;
  let chainId = 0;
  if(network === "devnet"){
    chainId = 3151908;
  }else{
    chainId = 17000;
  }
  let nonce = await wallet.getNonce();
  // nonce = 3
  console.log('nonce', nonce);
  // let slot = await getLatestSlot(chainId);
  let proposer = await getProposer(chainId);
  let count = 1;
  while(proposer === null){
    proposer = await getProposer(chainId);
    count ++;
    console.log(`tried ${count} times to find proposers, but not.`);

    await sleep(12)
  }
  console.log('proposer', proposer)
  // const proposer = {
  //   slot: slot +3,
  //   sidecar_url:"http://135.181.191.125:8017/",
  //   type:"bolt"
  // }
  const data  = await sendPreconfirmation(wallet, nonce, chainId, proposer);
}

main()