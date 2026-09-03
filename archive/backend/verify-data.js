import { duckdb } from './src/database/duckdb-connection.ts';

async function verifyData() {
  try {
    await duckdb.connect();
    
    // Check event count
    const events = await duckdb.query('SELECT COUNT(*) as count FROM events');
    console.log('Total events:', events.rows[0]?.count || 0);
    
    // Check recent events
    const eventList = await duckdb.query('SELECT id, name, date_time FROM events ORDER BY date_time LIMIT 10');
    console.log('\nFirst 10 events:');
    eventList.rows.forEach(e => console.log(`${e.id}: ${e.name} - ${e.date_time}`));
    
    // Check users
    const users = await duckdb.query('SELECT COUNT(*) as count FROM users');
    console.log('\nTotal users:', users.rows[0]?.count || 0);
    
    // Check venues
    const venues = await duckdb.query('SELECT COUNT(*) as count FROM venues');
    console.log('Total venues:', venues.rows[0]?.count || 0);
    
    await duckdb.close();
  } catch (error) {
    console.error('Error:', error);
  }
}

verifyData();