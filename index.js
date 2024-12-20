
import { Command } from 'commander';  
import { send_preconf_to_interstate_sidecar } from './preconf-cli/index.js';  

const program = new Command();

async function main() {

  program
    .command('send_interstate_sidecar')
    .description('Send preconfirmation to the interstate sidecar')
    .action(async () => {
      try {


      } catch (error) {
        console.error('Error sending preconfirmation:', error);
      }
    });


  await program.parseAsync(process.argv);
}

main();
