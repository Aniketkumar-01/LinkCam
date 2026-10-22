const https = require('https');
const fs = require('fs');

const options = {
  hostname: 'api.github.com',
  path: '/repos/Aniketkumar-01/LinkCam/actions/runs/39452351118/jobs',
  headers: { 'User-Agent': 'Node.js' }
};

https.get(options, (res) => {
  let data = '';
  res.on('data', (chunk) => data += chunk);
  res.on('end', () => {
    try {
      const json = JSON.parse(data);
      let output = '';
      for (const j of json.jobs || []) {
        output += `=== JOB: ${j.name} (Status: ${j.status}, Conclusion: ${j.conclusion}) ===\n`;
        for (const s of j.steps || []) {
          output += `  Step ${s.number}: ${s.name} -> ${s.conclusion}\n`;
        }
      }
      fs.writeFileSync('../ci_result.txt', output, 'utf8');
      console.log('CI RESULT SAVED');
    } catch (e) {
      console.error(e);
    }
  });
}).on('error', (e) => console.error(e));
