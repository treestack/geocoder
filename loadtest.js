import http from 'k6/http';
import { sleep, check } from 'k6';

export const options = {
  stages: [
    { duration: '5s', target: 10 },
    { duration: '25s', target: 20,
      thresholds: {
        checks: ['rate>0.9'],
      },},
  ],
};
export default function () {
  const res = http.get('http://localhost:5353?lat=50.9&lng=7.2&results=2');
  check(res, {
    'status was 200': (r) => r.status == 200
  });
  if (res.status !== 200) {
    console.error(`${res.status}: ${JSON.stringify(res.body)}`);
  }
  sleep(.1);
}
