@echo off
rem Runs the npm version pinned in package.json devDependencies, not the global one.
node "%~dp0node_modules\npm\bin\npm-cli.js" %*
