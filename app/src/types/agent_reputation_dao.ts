export type AgentReputationDao = {
  version: '0.1.0'
  name: 'agentreputation_dao'
  instructions: [
    {
      name: 'initialize'
      accounts: [
        { name: 'config', isMut: true, isSigner: false }
        { name: 'authority', isMut: true, isSigner: true }
        { name: 'systemProgram', isMut: false, isSigner: false }
      ]
      args: [
        { name: 'minStake', type: 'u64' }
        { name: 'decayRate', type: 'u64' }
        { name: 'rewardAmount', type: 'u64' }
      ]
    },
    {
      name: 'registerAgent'
      accounts: [
        { name: 'agentProfile', isMut: true, isSigner: false }
        { name: 'agent', isMut: true, isSigner: true }
        { name: 'systemProgram', isMut: false, isSigner: false }
      ]
      args: [
        { name: 'name', type: 'string' }
        { name: 'metadataUri', type: 'string' }
      ]
    },
    {
      name: 'completeTask'
      accounts: [
        { name: 'agentProfile', isMut: true, isSigner: false }
        { name: 'taskRecord', isMut: true, isSigner: false }
        { name: 'agent', isMut: true, isSigner: true }
        { name: 'config', isMut: false, isSigner: false }
        { name: 'systemProgram', isMut: false, isSigner: false }
      ]
      args: [
        { name: 'taskId', type: 'string' }
        { name: 'proofUri', type: 'string' }
      ]
    },
    {
      name: 'vouch'
      accounts: [
        { name: 'vouchRecord', isMut: true, isSigner: false }
        { name: 'voucherProfile', isMut: false, isSigner: false }
        { name: 'targetProfile', isMut: true, isSigner: false }
        { name: 'voucher', isMut: true, isSigner: true }
        { name: 'stakeVault', isMut: true, isSigner: false }
        { name: 'systemProgram', isMut: false, isSigner: false }
      ]
      args: [
        { name: 'targetAgent', type: 'publicKey' }
        { name: 'amount', type: 'u64' }
        { name: 'isPositive', type: 'bool' }
      ]
    },
    {
      name: 'decay'
      accounts: [
        { name: 'agentProfile', isMut: true, isSigner: false }
        { name: 'config', isMut: false, isSigner: false }
      ]
      args: []
    },
    {
      name: 'queryReputation'
      accounts: [{ name: 'agentProfile', isMut: false, isSigner: false }]
      args: []
    }
  ]
  accounts: {
    config: {
      authority: any
      minStake: any
      decayRate: any
      rewardAmount: any
      bump: number
    }
    agentProfile: {
      agent: any
      name: string
      reputationScore: any
      taskCount: any
      vouchCount: any
      stakedAmount: any
      lastActive: any
      bump: number
    }
    vouchRecord: {
      voucher: any
      target: any
      amount: any
      isPositive: boolean
      timestamp: any
      bump: number
    }
  }
  errors: [
    { code: 6000, name: 'Unauthorized', msg: 'Unauthorized' },
    { code: 6001, name: 'AlreadyRegistered', msg: 'Agent already registered' },
    { code: 6002, name: 'InsufficientStake', msg: 'Insufficient stake' },
    { code: 6003, name: 'SelfVouch', msg: 'Cannot vouch for yourself' },
    { code: 6004, name: 'TargetNotRegistered', msg: 'Target agent not registered' },
    { code: 6005, name: 'ReputationTooLow', msg: 'Reputation too low' }
  ]
}

export const IDL: AgentReputationDao = {
  version: '0.1.0',
  name: 'agentreputation_dao',
  instructions: [
    {
      name: 'initialize',
      accounts: [
        { name: 'config', isMut: true, isSigner: false },
        { name: 'authority', isMut: true, isSigner: true },
        { name: 'systemProgram', isMut: false, isSigner: false }
      ],
      args: [
        { name: 'minStake', type: 'u64' },
        { name: 'decayRate', type: 'u64' },
        { name: 'rewardAmount', type: 'u64' }
      ]
    },
    {
      name: 'registerAgent',
      accounts: [
        { name: 'agentProfile', isMut: true, isSigner: false },
        { name: 'agent', isMut: true, isSigner: true },
        { name: 'systemProgram', isMut: false, isSigner: false }
      ],
      args: [
        { name: 'name', type: 'string' },
        { name: 'metadataUri', type: 'string' }
      ]
    },
    {
      name: 'completeTask',
      accounts: [
        { name: 'agentProfile', isMut: true, isSigner: false },
        { name: 'taskRecord', isMut: true, isSigner: false },
        { name: 'agent', isMut: true, isSigner: true },
        { name: 'config', isMut: false, isSigner: false },
        { name: 'systemProgram', isMut: false, isSigner: false }
      ],
      args: [
        { name: 'taskId', type: 'string' },
        { name: 'proofUri', type: 'string' }
      ]
    },
    {
      name: 'vouch',
      accounts: [
        { name: 'vouchRecord', isMut: true, isSigner: false },
        { name: 'voucherProfile', isMut: false, isSigner: false },
        { name: 'targetProfile', isMut: true, isSigner: false },
        { name: 'voucher', isMut: true, isSigner: true },
        { name: 'stakeVault', isMut: true, isSigner: false },
        { name: 'systemProgram', isMut: false, isSigner: false }
      ],
      args: [
        { name: 'targetAgent', type: 'publicKey' },
        { name: 'amount', type: 'u64' },
        { name: 'isPositive', type: 'bool' }
      ]
    },
    {
      name: 'decay',
      accounts: [
        { name: 'agentProfile', isMut: true, isSigner: false },
        { name: 'config', isMut: false, isSigner: false }
      ],
      args: []
    },
    {
      name: 'queryReputation',
      accounts: [{ name: 'agentProfile', isMut: false, isSigner: false }],
      args: []
    }
  ],
  accounts: {
    config: {
      authority: {} as any,
      minStake: {} as any,
      decayRate: {} as any,
      rewardAmount: {} as any,
      bump: 0
    },
    agentProfile: {
      agent: {} as any,
      name: '',
      reputationScore: {} as any,
      taskCount: {} as any,
      vouchCount: {} as any,
      stakedAmount: {} as any,
      lastActive: {} as any,
      bump: 0
    },
    vouchRecord: {
      voucher: {} as any,
      target: {} as any,
      amount: {} as any,
      isPositive: false,
      timestamp: {} as any,
      bump: 0
    }
  },
  errors: [
    { code: 6000, name: 'Unauthorized', msg: 'Unauthorized' },
    { code: 6001, name: 'AlreadyRegistered', msg: 'Agent already registered' },
    { code: 6002, name: 'InsufficientStake', msg: 'Insufficient stake' },
    { code: 6003, name: 'SelfVouch', msg: 'Cannot vouch for yourself' },
    { code: 6004, name: 'TargetNotRegistered', msg: 'Target agent not registered' },
    { code: 6005, name: 'ReputationTooLow', msg: 'Reputation too low' }
  ]
}
