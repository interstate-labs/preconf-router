#! /usr/bin/env node

// import { program } from 'commander'
// import list from './commands/list'
import { ethers } from "ethers";
import { Command } from 'commander';
import { getProposer, getWallet, sendPreconfirmation, sleep, sendPreconfirmationToInterstateSidecar } from './lib/index.js';

const program = new Command();

async function main() {
  program
    .command('send')
    .description("My Node CLI")
    .requiredOption("-n, --network <type>", "devnet or holesky")
    .requiredOption("-p, --pvk <type>", "Private key to sign a transaction")
    .action(run);

  program
    .command('send_interstate_sidecar')
    .description("My Node CLI")
    .action(send_preconf_to_interstate_sidecar);
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
  
  let chainId = 1;
  if(network === "devnet"){
    chainId = 3151908;
  }else if(network === "holesky"){
    chainId = 17000;
  }
  let nonce = await wallet.getNonce();
  let proposer = await getProposer();
  let count = 1;
  while(proposer === undefined){
    proposer = await getProposer();
    count ++;
    console.log(`tried ${count} times to find proposers, but not.`);

    await sleep(12)
  }
  console.log('proposer', proposer)

  if(proposer)
  {
    const data  = await sendPreconfirmation(wallet, nonce, chainId, proposer);
  }
}


async function send_preconf_to_interstate_sidecar() {
  const chainId = 3151908;
  const wallet = await getWallet("devnet", "5d2344259f42259f82d2c140aa66102ba89b57b4883ee441a8b312622bd42491");
  let nonce = await wallet.getNonce();

  await sendPreconfirmationToInterstateSidecar(wallet,  nonce, chainId);
}

main()