const { exec } = require('child_process');
const fs = require('fs');
const dir = fs.opendirSync('./redis_commands');

function main() {
  let host = process.env.REDIS_HOST;
  let port = process.env.REDIS_PORT;
  let password = process.env.REDIS_PASSWORD;
  if (is_none(host) || is_none(port) || is_none(password)) {
    return;
  }
  let file;
  while ((file = dir.readSync()) !== null) {
    if (!file.isFile()) { continue; }
    let command = `redis-cli -h ${host} -p ${port} -a '${password}' < ${file.path}`;
    let commandDebug = `redis-cli -h ${host} -p ${port} -a 'XXXXX' < ${file.path}`;
    console.log(` Running command file: "${commandDebug}"`);
    exec(command, (err, stdout, stderr) => {
      if (err) {
        console.log(` Error: "${err}"`);
        // node couldn't execute the command
        return;
      }
    
      // the *entire* stdout and stderr (buffered)
      console.log(`stdout: ${stdout}`);
      console.log(`stderr: ${stderr}`);
    });
  }
}

function is_none(v) {
  return typeof(v) === "undefined" || v === null;
}
main();
