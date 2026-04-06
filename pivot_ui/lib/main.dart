import 'package:flutter/material.dart';
import 'package:pivot_ui/src/rust/frb_generated.dart'; // 🌉 The bridge
import 'package:pivot_ui/src/rust/api/simple.dart';

Future<void> main() async {
  // This turns on the bridge before the app starts drawing the screen
  await RustLib.init();
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Pivot Log',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
        useMaterial3: true,
      ),
      home: const PivotHomePage(),
    );
  }
}

class PivotHomePage extends StatefulWidget {
  const PivotHomePage({super.key});

  @override
  State<PivotHomePage> createState() => _PivotHomePageState();
}

class _PivotHomePageState extends State<PivotHomePage> {
  final _endedController = TextEditingController();
  final _startedController = TextEditingController();

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Pivot Log ⏱️'),
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
      ),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            TextField(
              controller: _endedController,
              decoration: const InputDecoration(
                labelText: 'Ended Tasks (e.g. 吃早餐)',
                border: OutlineInputBorder(),
              ),
            ),
            const SizedBox(height: 16),
            TextField(
              controller: _startedController,
              decoration: const InputDecoration(
                labelText: 'Started Tasks (e.g. rust)',
                border: OutlineInputBorder(),
              ),
            ),
            const SizedBox(height: 24),
            ElevatedButton.icon(
              onPressed: () async {
                // 1. Notice the 'async' keyword here
                // 2. Put the text into lists (since Rust expects Vec<String>)
                final endedList = [_endedController.text];
                final startedList = [_startedController.text];

                // 3. Load the database from the file
                var db = await PivotDb.load(path: "pivot_data.json");

                // 4. Send the tasks across the bridge! 🌉
                await db.logTransition(
                  endedTasks: endedList,
                  startedTasks: startedList,
                );

                // 5. Tell Rust to save the changes to the JSON file
                await db.save();

                // Show a quick pop-up message on the screen to confirm
                if (context.mounted) {
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(
                      content: Text('Tick saved to Rust backend! 💾'),
                    ),
                  );
                }

                _endedController.clear();
                _startedController.clear();
              },
              icon: const Icon(Icons.add_task),
              label: const Text('Log Tick'),
              style: ElevatedButton.styleFrom(
                minimumSize: const Size(double.infinity, 50),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
