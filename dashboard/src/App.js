import logo from './logo.svg';
import './App.css';

function App() {
  return (
    <div className="App">
        <h1>Holesky Proposer Statistics</h1>
        <div style="margin-bottom: 20px;">
            <a href="/mainnet" style="margin-right: 20px;">Mainnet</a>
            <a href="/holesky">Holesky</a>
        </div>
        <h2>Available Aggregated Proposers || Last Updated: <span class="count">{{ timestamp }}</span></h2>
        <h2>Total Proposers In Upcoming 32 Slots<span class="count">({{ bolt_proposers.len() + interstate_proposers.len()}})</span></h2>
        
        <div class="proposer-section">
            <table>
                <thead>
                    <tr>
                        <!-- <th>Type</th> enable to show which network proposers is under, not every network is on holesky rn--> 
                        <th>Type</th>
                        <th>Slot</th>
                        <th>Validator Index</th>
                        <th>URL</th>
                    </tr>
                </thead>
                <tbody>
                    {% for proposer in bolt_proposers %}
                    <tr class="bolt">
                        <td>Agg</td>
                        <td>{{ proposer.slot }}</td>
                        <td>{{ proposer.validator_index }}</td>
                        <td>{{ proposer.sidecar_url }}</td>
                    </tr>
                    {% endfor %}
                    {% for proposer in interstate_proposers %}
                    <tr class="interstate">
                        <td>Agg</td>
                        <td>{{ proposer.slot }}</td>
                        <td>{{ proposer.validator_index }}</td>
                        <td>{{ proposer.sidecar_url }}</td>
                    </tr>
                    {% endfor %}
                </tbody>
            </table>
        </div>
    </div>
  );
}

export default App;
