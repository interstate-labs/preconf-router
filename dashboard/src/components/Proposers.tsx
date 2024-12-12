export const Proposers = (props) => {
  const { proposers } = props;

  return (
    <>
      <table>
        <thead>
            <tr>
                <th>Type</th>
                <th>Slot</th>
                <th>Validator Index</th>
                <th>URL</th>
            </tr>
        </thead>
        <tbody>
          {proposers.map((proposer, index) => (
            <ProposerItem
              item={proposer}
            />
          ))}
        </tbody>
      </table>
    </>
  )
}

export const ProposerItem = (props) => {
  const { item } = props;

  return (
    <tr className="bolt">
        <td>Agg</td>
        <td>{ item.slot }</td>
        <td>{ item.validator_index }</td>
        <td>{ item.sidecar_url }</td>
    </tr>
  )
}
