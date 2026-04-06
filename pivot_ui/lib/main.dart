import 'package:flutter/material.dart';
import 'package:pivot_ui/src/rust/frb_generated.dart'; // 🌉 The bridge
import 'package:pivot_ui/src/rust/api/simple.dart';
import 'dart:io'; // Gives us the File tool to copy data
import 'package:file_picker/file_picker.dart';
import 'package:path_provider/path_provider.dart';

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
                // 1. Get the safe folder for your app database
                final directory = await getApplicationDocumentsDirectory();
                final dbPath = '${directory.path}/pivot_data.json';
                print(dbPath);

                // 2. Put the text into lists (since Rust expects Vec<String>)
                final endedList = [_endedController.text];
                final startedList = [_startedController.text];

                // 3. Load the database from the safe mobile path
                var db = await PivotDb.load(path: dbPath);

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
            const SizedBox(height: 16), // Adds space between the buttons
            OutlinedButton.icon(
              onPressed: () async {
                try {
                  // 1. Get the path to your internal database
                  final directory = await getApplicationDocumentsDirectory();
                  final filePath = '${directory.path}/pivot_data.json';
                  final currentFile = File(filePath);

                  if (!await currentFile.exists()) {
                    throw "Database file not found at $filePath";
                  }

                  // 2. Read the bytes
                  final bytes = await currentFile.readAsBytes();

                  // 3. Ask the OS for a save location
                  String? savePath = await FilePicker.platform.saveFile(
                    dialogTitle: 'Save your database backup',
                    fileName: 'pivot_data_backup.json',
                    type: FileType.custom,
                    allowedExtensions: ['json'],
                    bytes:
                        bytes, // This saves the file automatically ON ANDROID
                  );

                  // 4. FIX FOR WINDOWS: If we have a path but no file was created
                  if (savePath != null) {
                    final exportFile = File(savePath);

                    // On Windows, 'saveFile' doesn't write the bytes, so we do it manually:
                    if (!Platform.isAndroid && !Platform.isIOS) {
                      await exportFile.writeAsBytes(bytes);
                    }

                    if (context.mounted) {
                      ScaffoldMessenger.of(context).showSnackBar(
                        const SnackBar(content: Text('Export successful! 📁')),
                      );
                    }
                  }
                } catch (e) {
                  if (context.mounted) {
                    ScaffoldMessenger.of(context).showSnackBar(
                      SnackBar(content: Text('Export failed: $e ❌')),
                    );
                  }
                }
              },
              icon: const Icon(Icons.download),
              label: const Text('Export Data'),
              style: OutlinedButton.styleFrom(
                minimumSize: const Size(double.infinity, 50),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
