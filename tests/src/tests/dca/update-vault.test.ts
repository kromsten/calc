import { Context } from 'mocha';
import { map } from 'ramda';
import { execute } from '../../shared/cosmwasm';
import { EventData } from '../../types/dca/response/get_events';
import { Vault } from '../../types/dca/response/get_vault';
import { createVault } from '../helpers';
import { expect } from '../shared.test';
import dayjs from 'dayjs';

describe('when updating a vault', () => {
  describe('with a new time interval', async () => {
    let vaultBeforeExecution: Vault;
    let vaultAfterExecution: Vault;
    let eventPayloadsAfterExecution: EventData[];

    before(async function (this: Context) {
      const vaultId = await createVault(this, {
        time_interval: 'daily',
      });

      vaultBeforeExecution = (
        await this.cosmWasmClient.queryContractSmart(this.dcaContractAddress, {
          get_vault: {
            vault_id: vaultId,
          },
        })
      ).vault;

      await execute(this.userCosmWasmClient, this.userWalletAddress, this.dcaContractAddress, {
        update_vault: {
          vault_id: vaultBeforeExecution.id,
          time_interval: { custom: { seconds: 60 } },
        },
      });

      vaultAfterExecution = (
        await this.cosmWasmClient.queryContractSmart(this.dcaContractAddress, {
          get_vault: {
            vault_id: vaultId,
          },
        })
      ).vault;

      eventPayloadsAfterExecution = map(
        (event) => event.data,
        (
          await this.cosmWasmClient.queryContractSmart(this.dcaContractAddress, {
            get_events_by_resource_id: { resource_id: vaultId },
          })
        ).events,
      );
    });

    it('updates the time interval', () =>
      expect(vaultAfterExecution.time_interval).to.eql({ custom: { seconds: 60 } }));

    it('updates the trigger target time', () => {
      expect('time' in vaultBeforeExecution.trigger && vaultBeforeExecution.trigger.time.target_time).to.not.eql(
        'time' in vaultAfterExecution.trigger && vaultAfterExecution.trigger.time.target_time,
      );
      expect(
        dayjs(
          Number('time' in vaultBeforeExecution.trigger && vaultBeforeExecution.trigger.time.target_time) / 1000000,
        ).toISOString(),
      ).to.equal(
        dayjs(Number('time' in vaultAfterExecution.trigger && vaultAfterExecution.trigger.time.target_time) / 1000000)
          .add(23, 'hours')
          .add(59, 'minutes')
          .toISOString(),
      );
    });

    it('publishes a vault updated event', () => {
      expect(eventPayloadsAfterExecution).to.deep.include({
        dca_vault_updated: {
          updates: [
            {
              field: 'time_interval',
              old_value: 'Daily',
              new_value: 'Custom:60',
            },
            {
              field: 'trigger',
              old_value: `Time { target_time: Timestamp(Uint64(${
                dayjs(Number(vaultBeforeExecution.created_at) / 1000000)
                  .add(1, 'day')
                  .unix() * 1000000000
              })) }`,
              new_value: `Time { target_time: Timestamp(Uint64(${
                dayjs(Number(vaultBeforeExecution.created_at) / 1000000)
                  .add(1, 'minute')
                  .unix() * 1000000000
              })) }`,
            },
          ],
        },
      });
    });
  });
});
