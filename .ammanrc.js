// @ts-check
'use strict';
const path = require('path');
const accountProviders = require('./packages/sdk/dist/generated/accounts');

const localDeployDir = path.join(__dirname, 'program', 'target', 'deploy');
const MY_PROGRAM_ID = require("./packages/sdk/idl/sprite_manager.json").metadata.address;

function localDeployPath(programName) {
    return path.join(localDeployDir, `${programName}.so`);
}

const programs = [
    {
        label: 'sprite_manager',
        programId: MY_PROGRAM_ID,
        deployPath: localDeployPath('sprite_manager')
    },
];

const accounts = [
    {
        label: 'Token Metadata Program',
        accountId:'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s',
        // marking executable as true will cause Amman to pull the executable data account as well automatically
        executable: true,
    },
];

const validator = {
    programs,
    // The accounts below is commented out. Uncomment if you want to pull remote accounts. Check Amman docs for more info
    accounts,
    verifyFees: false,
    limitLedgerSize: 10000000,
};

module.exports = {
    validator,
    relay: {
        accountProviders,

    },
};